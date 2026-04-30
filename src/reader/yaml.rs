use std::path::PathBuf;
use serde::Deserialize;
use anyhow::{bail, Result};
use regex::Regex;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ColumnRule {
    Integer { min: Option<u64>, max: Option<u64> },
    Float { min: Option<f64>, max: Option<f64> },
    Text { pattern: Option<String> },
    Date { before: Option<String>, after: Option<String> },
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
        for (name, config) in &self.columns {
            match &config.rule {
                ColumnRule::Integer { min, max } => {
                    // min y max pueden ser None, lo que significa que no hay límite inferior o superior respectivamente. Si ambos son Some, entonces min debe ser menor o igual a max.
                    if let (Some(min_val), Some(max_val)) = (min, max) {
                        if min_val > max_val {
                            bail!("En la columna '{}', el valor mínimo ({}) no puede ser mayor que el valor máximo ({}).", name, min_val, max_val);
                        }
                    }
                }
                ColumnRule::Float { min, max } => {
                    // min y max pueden ser None, lo que significa que no hay límite inferior o superior respectivamente. Si ambos son Some, entonces min debe ser menor o igual a max.
                    if let (Some(min_val), Some(max_val)) = (min, max) {
                        if min_val > max_val {
                            bail!("En la columna '{}', el valor mínimo ({}) no puede ser mayor que el valor máximo ({}).", name, min_val, max_val);
                        }
                    }
                }
                ColumnRule::Date { before, after     } => {
                    // Si before es Some, verificar que sea una fecha válida en formato YYYY-MM-DD.
                    if let Some(date_str) = before {
                        if let Err(e) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            bail!("En la columna '{}', la fecha 'before' '{}' no es una fecha válida en formato YYYY-MM-DD: {}", name, date_str, e);
                        }
                    }
                    // Si after es Some, verificar que sea una fecha válida en formato YYYY-MM-DD.
                    if let Some(date_str) = after {
                        if let Err(e) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            bail!("En la columna '{}', la fecha 'after' '{}' no es una fecha válida en formato YYYY-MM-DD: {}", name, date_str, e);
                        }
                    }

                    if before.is_some() && after.is_some() {
                        let before_date = NaiveDate::parse_from_str(before.as_ref().unwrap(), "%Y-%m-%d")?;
                        let after_date = NaiveDate::parse_from_str(after.as_ref().unwrap(), "%Y-%m-%d")?;
                        if before_date < after_date {
                            bail!("En la columna '{}', la fecha 'before' ({}) no puede ser anterior a la fecha 'after' ({}).", name, before.as_ref().unwrap(), after.as_ref().unwrap());
                        }
                    }
                }
                ColumnRule::Text { pattern } => {
                    // Si pattern es Some, verificar que sea una expresión regular válida. Si es None, no hay restricción de patrón.
                    if let Some(pat) = pattern {
                        // Verificar que el patrón sea una expresión regular válida
                        if let Err(e) = Regex::new(pat) {
                            bail!("En la columna '{}', el patrón de texto '{}' no es una expresión regular válida: {}", name, pat, e);
                        }
                    }
                }
                ColumnRule::Email => {}
            }
        }
        Ok(())
    }
}

pub fn load(path: &PathBuf) -> Result<Rules> {
    let file = std::fs::File::open(path)?;
    let rules: Rules = serde_yaml::from_reader(file)?;
    rules.validate()?;
    Ok(rules)
}