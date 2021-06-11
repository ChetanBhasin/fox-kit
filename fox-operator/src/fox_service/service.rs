use fox_k8s_crds::fox_service::FoxServiceSpec;
use k8s_openapi::api::core::v1::{Service, ServicePort, ServiceSpec};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{DeleteParams, ObjectMeta, PostParams};
use kube::{Api, Client, Error};

fn build_service(fs: &FoxServiceSpec, namespace: &str) -> Service {
    let ports = fs.http_ingress.as_ref().map(|ingress| {
        ingress
            .iter()
            .map(|ingress| ServicePort {
                port: ingress.port,
                protocol: None,
                target_port: Some(IntOrString::Int(ingress.port)),
                ..ServicePort::default()
            })
            .collect()
    });
    Service {
        metadata: ObjectMeta {
            annotations: None,
            labels: None,
            name: Some(fs.name.to_owned()),
            namespace: Some(namespace.to_owned()),
            owner_references: None,
            ..ObjectMeta::default()
        },
        spec: Some(ServiceSpec {
            ports,
            selector: None,
            ..ServiceSpec::default()
        }),
        ..Service::default()
    }
}

/// Creates a new service for the contianers that expose ports
///
/// # Arguments
/// - `client` - A Kubernetes client to create the service with.
/// - `fs` - Fox service specification
/// - `name` - Name of the service to be created
/// - `namespace` - Namespace to create the Kubernetes Service in.
///
/// Note: It is assumed the resource does not already exists for simplicity. Returns an `Error` if it does.
pub async fn create_service(
    client: Client,
    fs: &FoxServiceSpec,
    namespace: &str,
) -> Result<Service, Error> {
    // Definition of the service. Alternatively, a YAML representation could be used as well.
    let service: Service = build_service(fs, namespace);

    // Create the service defined above
    let service_api: Api<Service> = Api::namespaced(client, namespace);
    service_api.create(&PostParams::default(), &service).await
}

/// Deletes an existing service.
///
/// # Arguments:
/// - `client` - A Kubernetes client to delete the Service with
/// - `name` - Name of the service to delete
/// - `namespace` - Namespace the existing service resides in
///
/// Note: It is assumed the service exists for simplicity. Otherwise returns an Error.
pub async fn delete_service(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<Service> = Api::namespaced(client, namespace);
    api.delete(name, &DeleteParams::default()).await?;
    Ok(())
}
