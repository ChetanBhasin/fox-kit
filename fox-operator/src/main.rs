use futures::stream::StreamExt;
use kube::{api::ListParams, client::Client, Api};
use kube::{Resource, ResourceExt};
use kube_runtime::controller::{Context, ReconcilerAction};
use kube_runtime::Controller;
use tokio::time::Duration;

use fox_k8s_crds::fox_service::*;

mod finalizer;
mod fox_service;

#[tokio::main]
async fn main() {
    // First, a Kubernetes client must be obtained using the `kube` crate
    // The client will later be moved to the custom controller
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    // Preparation of resources used by the `kube_runtime::Controller`
    let crd_api: Api<FoxService> = Api::all(kubernetes_client.clone());
    let context: Context<ContextData> = Context::new(ContextData::new(kubernetes_client.clone()));

    // The controller comes from the `kube_runtime` crate and manages the reconciliation process.
    // It requires the following information:
    // - `kube::Api<T>` this controller "owns". In this case, `T = FoxService`, as this controller owns the `FoxService` resource,
    // - `kube::api::ListParams` to select the `FoxService` resources with. Can be used for FoxService filtering `FoxService` resources before reconciliation,
    // - `reconcile` function with reconciliation logic to be called each time a resource of `FoxService` kind is created/updated/deleted,
    // - `on_error` function to call whenever reconciliation fails.
    Controller::new(crd_api.clone(), ListParams::default())
        .run(reconcile, on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(fox_serv_res) => {
                    println!("Reconciliation successful. Resource: {:?}", fox_serv_res);
                }
                Err(reconciliation_err) => {
                    eprintln!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

/// Context injected with each `reconcile` and `on_error` method invocation.
struct ContextData {
    /// Kubernetes client to make Kubernetes API requests with. Required for K8S resource management.
    client: Client,
}

impl ContextData {
    /// Constructs a new instance of ContextData.
    ///
    /// # Arguments:
    /// - `client`: A Kubernetes client to make Kubernetes REST API requests with. Resources
    /// will be created and deleted with this client.
    pub fn new(client: Client) -> Self {
        ContextData { client }
    }
}

/// Action to be taken upon an `FoxService` resource during reconciliation
enum Action {
    /// Create the subresources, this includes spawning `n` pods with FoxService service
    Create,
    /// Delete all subresources created in the `Create` phase
    Delete,
    /// This `FoxService` resource is in desired state and requires no actions to be taken
    NoOp,
}

async fn reconcile(
    fox_svc: FoxService,
    context: Context<ContextData>,
) -> Result<ReconcilerAction, Error> {
    let client: Client = context.get_ref().client.clone(); // The `Client` is shared -> a clone from the reference is obtained

    // The resource of `FoxService` kind is required to have a namespace set. However, it is not guaranteed
    // the resource will have a `namespace` set. Therefore, the `namespace` field on object's metadata
    // is optional and Rust forces the programmer to check for it's existence first.
    let namespace: String = match fox_svc.namespace() {
        None => {
            // If there is no namespace to deploy to defined, reconciliation ends with an error immediately.
            return Err(Error::UserInputError(
                "Expected FoxService resource to be namespaced. Can't deploy to an unknown namespace."
                    .to_owned(),
            ));
        }
        // If namespace is known, proceed. In a more advanced version of the operator, perhaps
        // the namespace could be checked for existence first.
        Some(namespace) => namespace,
    };

    // Performs action as decided by the `determine_action` function.
    return match determine_action(&fox_svc) {
        Action::Create => {
            // Creates a deployment with `n` FoxService service pods, but applies a finalizer first.
            // Finalizer is applied first, as the operator might be shut down and restarted
            // at any time, leaving subresources in intermediate state. This prevents leaks on
            // the `FoxService` resource deletion.
            let name = fox_svc.name(); // Name of the FoxService resource is used to name the subresources as well.

            // Apply the finalizer first. If that fails, the `?` operator invokes automatic conversion
            // of `kube::Error` to the `Error` defined in this crate.
            finalizer::add(client.clone(), &name, &namespace).await?;
            // Invoke creation of a Kubernetes built-in resource named deployment with `n` fox service pods.
            fox_service::deploy(
                &fox_svc.spec,
                client,
                &fox_svc.name(),
                fox_svc.spec.replicas,
                &namespace,
            )
            .await?;
            Ok(ReconcilerAction {
                // Finalizer is added, deployment is deployed, re-check in 10 seconds.
                requeue_after: Some(Duration::from_secs(10)),
            })
        }
        Action::Delete => {
            // Deletes any subresources related to this `FoxService` resources. If and only if all subresources
            // are deleted, the finalizer is removed and Kubernetes is free to remove the `FoxService` resource.

            //First, delete the deployment. If there is any error deleting the deployment, it is
            // automatically converted into `Error` defined in this crate and the reconciliation is ended
            // with that error.
            // Note: A more advanced implementation would for the Deployment's existence.
            fox_service::delete(client.clone(), &fox_svc.name(), &namespace).await?;

            // Once the deployment is successfully removed, remove the finalizer to make it possible
            // for Kubernetes to delete the `FoxService` resource.
            finalizer::delete(client, &fox_svc.name(), &namespace).await?;
            Ok(ReconcilerAction {
                requeue_after: None, // Makes no sense to delete after a successful delete, as the resource is gone
            })
        }
        Action::NoOp => Ok(ReconcilerAction {
            // The resource is already in desired state, do nothing and re-check after 10 seconds
            requeue_after: Some(Duration::from_secs(10)),
        }),
    };
}

/// Resources arrives into reconciliation queue in a certain state. This function looks at
/// the state of given `FoxService` resource and decides which actions needs to be performed.
/// The finite set of possible actions is represented by the `Action` enum.
///
/// # Arguments
/// - `fox_svc`: A reference to `FoxService` being reconciled to decide next action upon.
fn determine_action(fox_svc: &FoxService) -> Action {
    return if fox_svc.meta().deletion_timestamp.is_some() {
        Action::Delete
    } else if fox_svc.meta().finalizers.is_none() {
        Action::Create
    } else {
        Action::NoOp
    };
}

/// Actions to be taken when a reconciliation fails - for whatever reason.
/// Prints out the error to `stderr` and requeues the resource for another reconciliation after
/// five seconds.
///
/// # Arguments
/// - `error`: A reference to the `kube::Error` that occurred during reconciliation.
/// - `_context`: Unused argument. Context Data "injected" automatically by kube-rs.
fn on_error(error: &Error, _context: Context<ContextData>) -> ReconcilerAction {
    eprintln!("Reconciliation error:\n{:?}", error);
    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(5)),
    }
}

/// All errors possible to occur during reconciliation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or FoxService resource definition, typically missing fields.
    #[error("Invalid FoxService CRD: {0}")]
    UserInputError(String),
}
