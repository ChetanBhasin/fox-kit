use schemars::schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Names {
    pub kind: String,
    pub plural: String,
    pub singular: String,
    pub short_names: Vec<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub spec: Schema,
    pub status: Option<Schema>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: Properties,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPISchema {
    #[serde(rename = "openAPIV3Schema")]
    pub open_apiv3schema: ObjectSchema,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub name: String,
    pub served: bool,
    pub storage: bool,
    pub schema: OpenAPISchema,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
    pub group: String,
    pub names: Names,
    pub scope: String,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KubernetesCRD {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}
