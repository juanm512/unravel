use std::path::PathBuf;
use serde::Deserialize;
use anyhow::{bail, Result};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ColumnRule {
    Numeric { min: Option<f64>, max: Option<f64> },
    Text { pattern: Option<String> },
    Date { before: Option<String> },
    Email,
}
impl Default for ColumnRule {
    fn default() -> Self {
        ColumnRule::Text { pattern: None }
    }
}

#[derive(Debug, Deserialize)]
pub struct ColumnsConfig {
    #[serde(default)]
    pub rule: ColumnRule,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub unique: bool,

}
impl Default for ColumnsConfig {
    fn default() -> Self {
        Self {
            rule: ColumnRule::default(),
            required: false,
            unique: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Rules {
    pub columns: std::collections::HashMap<String, ColumnsConfig>,
}

impl Rules {
    pub fn validate(&self) -> Result<()> {
        // ¿qué casos incoherentes chequeás?

        Ok(())
    }
}

pub fn load(path: &PathBuf) -> Result<Rules> {
    let file = std::fs::File::open(path)?;
    let rules: Rules = serde_yaml::from_reader(file)?;
    Ok(rules)
}