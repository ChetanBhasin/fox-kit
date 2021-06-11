use fox_k8s_crds::fox_service::FoxServiceSpec;

fn main() {
    let pwd = std::env::var("PWD").expect("Could not get PWD from environment");
    let fox_service_crd = FoxServiceSpec::kubernetes_crd();
    let schema_string =
        serde_yaml::to_string(&fox_service_crd).expect("Could not get schema from RootSchema");
    std::fs::write(format!("{}/foxservices.cbopt.com.yaml", pwd), schema_string)
        .expect("Could not write the JSON file");
}
