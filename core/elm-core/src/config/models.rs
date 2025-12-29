use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelV1 {
    pub schema: String,
    pub name: String,
    pub description: String,
    pub priority: i64,
    pub defaults: ChannelDefaults,
    pub manifests: HashMap<String, String>,
    pub constraints: Option<ChannelConstraints>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelDefaults {
    pub manifest: String,
    pub engine: String,
    pub settings_preset: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelConstraints {
    pub min_vulkan: Option<String>,
    pub gpu_vendors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineV1 {
    pub schema: String,
    pub id: String,
    #[serde(rename = "type")]
    pub engine_type: String,
    pub source: EngineSource,
    pub layout: EngineLayout,
    pub sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineSource {
    pub kind: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineLayout {
    pub proton_root_subdir: String,
    pub runner: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestV1 {
    pub schema: String,
    pub id: String,
    pub display_name: String,
    pub installer: Installer,
    pub engine: ManifestEngineRef,
    pub runtime: RuntimeConfig,
    pub env: Option<EnvConfig>,
    pub launch: LaunchConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Installer {
    #[serde(rename = "type")]
    pub installer_type: String,
    pub source: InstallerSource,
    pub install_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallerSource {
    pub url: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestEngineRef {
    #[serde(rename = "ref")]
    pub engine_ref: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub wineprefix_layout: String,
    pub dx: DxConfig,
    pub components: ComponentsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DxConfig {
    pub preferred: String,
    pub allow_dx12: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentsConfig {
    pub dxvk: ToggleConfig,
    pub vkd3d: ToggleConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToggleConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvConfig {
    pub base: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaunchConfig {
    pub entrypoints: Vec<Entrypoint>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Entrypoint {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub path: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileV1 {
    pub schema: String,
    pub name: String,
    pub channel: String,
    pub manifest: String,
    pub engine: String,
    pub overrides: serde_json::Value,
}
