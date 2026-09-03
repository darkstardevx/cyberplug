use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SchemaField {
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub label: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default, rename = "defaultValue")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BarWidget {
    #[serde(default, rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub defaults: Option<serde_json::Value>,
    #[serde(default)]
    pub schema: Vec<SchemaField>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default, rename = "firstParty")]
    pub first_party: Option<bool>,
    pub version: Option<String>,
    pub author: Option<String>,
    #[serde(default, rename = "barWidget")]
    pub bar_widget: Option<BarWidget>,
}

impl Plugin {
    pub fn schema(&self) -> &[SchemaField] {
        self.bar_widget
            .as_ref()
            .map(|bw| bw.schema.as_slice())
            .unwrap_or(&[])
    }
}
