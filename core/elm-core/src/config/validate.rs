use anyhow::{anyhow, Context, Result};
use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn validate_json_against_schema(json: &Value, schema_path: &Path) -> Result<()> {
    let schema_str = fs::read_to_string(schema_path)
        .with_context(|| format!("reading schema: {}", schema_path.display()))?;
    let schema_json: Value = serde_json::from_str(&schema_str)
        .with_context(|| format!("parsing schema json: {}", schema_path.display()))?;
    let compiled = Validator::new(&schema_json)
        .map_err(|e| anyhow!("schema compile error {}: {e}", schema_path.display()))?;
    if let Err(errors) = compiled.validate(json) {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        return Err(anyhow!("schema validation failed:\n- {}", msgs.join("\n- ")));
    }
    Ok(())
}
