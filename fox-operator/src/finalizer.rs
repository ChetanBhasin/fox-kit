use fox_k8s_crds::fox_service::*;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error};
use serde_json::{json, Value};

/// Adds a finalizer record into an `FoxService` kind of resource. If the finalizer already exists,
/// this action has no effect.
///
/// # Arguments:
/// - `client` - Kubernetes client to modify the `FoxService` resource with.
/// - `name` - Name of the `FoxService` resource to modify. Existence is not verified
/// - `namespace` - Namespace where the `FoxService` resource with given `name` resides.
///
/// Note: Does not check for resource's existence for simplicity.
pub async fn add(client: Client, name: &str, namespace: &str) -> Result<FoxService, Error> {
    let api: Api<FoxService> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": ["foxservices.cbopt.com"]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}

/// Removes all finalizers from an `FoxService` resource. If there are no finalizers already, this
/// action has no effect.
///
/// # Arguments:
/// - `client` - Kubernetes client to modify the `FoxService` resource with.
/// - `name` - Name of the `FoxService` resource to modify. Existence is not verified
/// - `namespace` - Namespace where the `FoxService` resource with given `name` resides.
///
/// Note: Does not check for resource's existence for simplicity.
pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<FoxService, Error> {
    let api: Api<FoxService> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}
