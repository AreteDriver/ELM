use super::models::*;
use super::validate::validate_json_against_schema;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

fn read_json(path: &Path) -> Result<Value> {
    let s = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let v: Value = serde_json::from_str(&s).with_context(|| format!("parsing {}", path.display()))?;
    Ok(v)
}

pub fn load_channel(path: &Path, schemas_dir: &Path) -> Result<ChannelV1> {
    let v = read_json(path)?;
    validate_json_against_schema(&v, &schemas_dir.join("elm.channel.v1.schema.json"))?;
    Ok(serde_json::from_value(v)?)
}

pub fn load_engine(path: &Path, schemas_dir: &Path) -> Result<EngineV1> {
    let v = read_json(path)?;
    validate_json_against_schema(&v, &schemas_dir.join("elm.engine.v1.schema.json"))?;
    Ok(serde_json::from_value(v)?)
}

pub fn load_manifest(path: &Path, schemas_dir: &Path) -> Result<ManifestV1> {
    let v = read_json(path)?;
    validate_json_against_schema(&v, &schemas_dir.join("elm.manifest.v1.schema.json"))?;
    Ok(serde_json::from_value(v)?)
}

pub fn load_profile(path: &Path, schemas_dir: &Path) -> Result<ProfileV1> {
    let v = read_json(path)?;
    validate_json_against_schema(&v, &schemas_dir.join("elm.profile.v1.schema.json"))?;
    Ok(serde_json::from_value(v)?)
}
