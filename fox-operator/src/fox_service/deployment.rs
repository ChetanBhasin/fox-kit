use fox_k8s_crds::fox_service::*;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::EnvVar;
use k8s_openapi::api::core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec};
use kube::api::{DeleteParams, ObjectMeta, PostParams};
use kube::{Api, Client, Error};

fn build_deployment(fs: &FoxServiceSpec, namespace: &str) -> Deployment {
    let containers = fs
        .containers
        .iter()
        .map(|container| {
            let ports = container.ports.as_ref().map(|ports| {
                ports
                    .iter()
                    .map(|(host, container)| ContainerPort {
                        container_port: container.to_owned(),
                        host_port: Some(host.to_owned()),
                        ..ContainerPort::default()
                    })
                    .collect()
            });
            let env = container.env.as_ref().map(|env| {
                env.iter()
                    .map(|(key, value)| EnvVar {
                        name: key.to_owned(),
                        value: Some(value.to_owned()),
                        ..EnvVar::default()
                    })
                    .collect()
            });
            Container {
                name: container.name.to_owned(),
                image: Some(container.image.to_owned()),
                image_pull_policy: Some("ALways".to_string()),
                args: container.args.clone(),
                env,
                ports,
                ..Container::default()
            }
        })
        .collect();
    Deployment {
        metadata: ObjectMeta {
            name: Some(fs.name.to_owned()),
            namespace: Some(namespace.to_owned()),
            ..ObjectMeta::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(fs.replicas),
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers,
                    ..PodSpec::default()
                }),
                metadata: Some(ObjectMeta {
                    ..ObjectMeta::default()
                }),
                ..PodTemplateSpec::default()
            },
            ..DeploymentSpec::default()
        }),
        ..Deployment::default()
    }
}

/// Creates a new deployment of `n` pods with the `inanimate/echo-server:latest` docker image inside,
/// where `n` is the number of `replicas` given.
///
/// # Arguments
/// - `client` - A Kubernetes client to create the deployment with.
/// - `fs` - Fox service specification
/// - `name` - Name of the deployment to be created
/// - `namespace` - Namespace to create the Kubernetes Deployment in.
///
/// Note: It is assumed the resource does not already exists for simplicity. Returns an `Error` if it does.
pub async fn create_deployment(
    client: Client,
    fs: &FoxServiceSpec,
    namespace: &str,
) -> Result<Deployment, Error> {
    // Definition of the deployment. Alternatively, a YAML representation could be used as well.
    let deployment: Deployment = build_deployment(fs, namespace);

    // Create the deployment defined above
    let deployment_api: Api<Deployment> = Api::namespaced(client, namespace);
    deployment_api
        .create(&PostParams::default(), &deployment)
        .await
}

/// Deletes an existing deployment.
///
/// # Arguments:
/// - `client` - A Kubernetes client to delete the Deployment with
/// - `name` - Name of the deployment to delete
/// - `namespace` - Namespace the existing deployment resides in
///
/// Note: It is assumed the deployment exists for simplicity. Otherwise returns an Error.
pub async fn delete_deployment(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<Deployment> = Api::namespaced(client, namespace);
    api.delete(name, &DeleteParams::default()).await?;
    Ok(())
}
