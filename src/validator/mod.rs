
use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use regex::Regex;

use crate::{reader::yaml::ColumnRule, yaml::Rules};

pub struct ValidationError {
    pub column: String,
    pub row: usize,
    pub message: String,
    pub value: Option<String>,
}

pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub rows_with_errors: HashSet<usize>,

    pub total_rows: usize,
    pub total_errors: usize,
}
impl ValidationReport {
    pub fn new() -> Self {
        ValidationReport {
            errors: Vec::new(),
            rows_with_errors: HashSet::new(),
            total_rows: 0,
            total_errors: 0,
        }
    }

    pub fn add_error(&mut self, row: usize, error: ValidationError) {
        self.errors.push(error);
        self.total_errors += 1;
        self.rows_with_errors.insert(row);
    }

    pub fn set_total_rows(&mut self, total: usize) {
        self.total_rows = total;
    }
}

pub fn validate(
    headers: &[String],
    records: &[csv::StringRecord],
    rules: &Rules,
) -> ValidationReport {
    let mut report = ValidationReport::new();

    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    let mut patterns_regexs: HashMap<String, Regex> = HashMap::new();
    rules.columns.iter().for_each(|rule: (&String, &crate::reader::yaml::ColumnsConfig)| {
        if let ColumnRule::Text { pattern: Some(p) } = &rule.1.rule {
            patterns_regexs.insert(rule.0.clone(), Regex::new(p).unwrap());
        }
    });
    let mut unique_values: HashMap<&str, HashMap<String, Vec<usize>>> = HashMap::new();

    for (i, record) in records.iter().enumerate() {
        // iteramos sobre cada regla y validamos la columna correspondiente
        for rule in &rules.columns {
            let column_name = rule.0;
            let column_config = &rule.1;

            if let Some(column_index) = headers.iter().position(|h| h == column_name) {
                let value = record.get(column_index).unwrap_or("");

                if column_config.required && value.trim().is_empty() {
                    report.add_error(i + 1, ValidationError {
                        column: column_name.clone(),
                        row: i + 1,
                        message: format!("Value in column '{}' is required", column_name),
                        value: Some(value.to_string()),
                    });
                    continue; // Si es requerido y está vacío, no hace falta validar el formato
                }
                if value.trim().is_empty() {
                    continue; // no-required y vacío → skip tipo
                }

                if column_config.unique {
                    unique_values.entry(column_name).or_insert_with(HashMap::new).entry(value.to_string()).or_insert_with(Vec::new).push(i + 1);
                }

                let error_message = match &column_config.rule {
                    ColumnRule::Text { pattern } => {
                        if let Some(p) = pattern {
                            if !validate_pattern(value, patterns_regexs.get(column_name).unwrap()) {
                                Some(format!("Value '{}' does not match pattern '{}'", value, p))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    ColumnRule::Integer { min, max } => {
                        if !validate_integer(value, *min, *max) {
                            Some(format!("Value '{}' is not a valid integer within the specified range", value))
                        } else {
                            None
                        }
                    }
                    ColumnRule::Float { min, max } => {
                        if !validate_float(value, *min, *max) {
                            Some(format!("Value '{}' is not a valid float within the specified range", value))
                        } else {
                            None
                        }
                    }
                    ColumnRule::Date { before, after } => {
                        if !validate_date(value, before.as_deref(), after.as_deref()) {
                            Some(format!("Value '{}' is not a valid date within the specified range", value))
                        } else {
                            None
                        }
                    }
                    ColumnRule::Email => {
                        if !validate_email(value, &email_regex) {
                            Some(format!("Value '{}' is not a valid email address", value))
                        } else {
                            None
                        }
                    }
                };

                if let Some(message) = error_message {
                    report.add_error(i + 1, ValidationError {
                        column: column_name.clone(),
                        row: i + 1,
                        message,
                        value: Some(value.to_string()),
                    });
                }
            }
        }
    }

    // Validar unicidad después de procesar todas las filas
    for (column_name, values_map) in unique_values {
        for (value, rows) in values_map {
            if rows.len() > 1 {
                let Some((_first, elements)) = rows.split_first() else { continue };
                for row in elements {
                    report.add_error(*row, ValidationError {
                        column: column_name.to_string(),
                        row: *row,
                        message: format!("Value '{}' is not unique in column '{}'", value, column_name),
                        value: Some(value.clone()),
                    });
                }
            }
        }
    }

    report.set_total_rows(records.len());
    report
}


pub fn validate_email(email: &str, email_regex: &Regex) -> bool {
    // Un regex simple para validar emails. No es perfecto, pero cubre la mayoría de los casos comunes.
    if email.is_empty() {
        return false;
    }

    email_regex.is_match(email)
}

pub fn validate_pattern(value: &str, pattern: &Regex) -> bool {
    return pattern.is_match(value)
}

pub fn validate_integer(value: &str, min: Option<u64>, max: Option<u64>) -> bool {
    if let Ok(num) = value.parse::<u64>() {
        if let Some(min_val) = min {
            if num < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if num > max_val {
                return false;
            }
        }
        return true;
    }
    false
}

pub fn validate_float(value: &str, min: Option<f64>, max: Option<f64>) -> bool {
    if let Ok(num) = value.parse::<f64>() {
        if let Some(min_val) = min {
            if num < min_val {
                return false;
            }
        }
        if let Some(max_val) = max {
            if num > max_val {
                return false;
            }
        }
        return true;
    }
    false
}

pub fn validate_date(value: &str, before: Option<&str>, after: Option<&str>) -> bool {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(before_str) = before {
            if let Ok(before_date) = NaiveDate::parse_from_str(before_str, "%Y-%m-%d") {
                if date >= before_date {
                    return false;
                }
            }
        }
        if let Some(after_str) = after {
            if let Ok(after_date) = NaiveDate::parse_from_str(after_str, "%Y-%m-%d") {
                if date <= after_date {
                    return false;
                }
            }
        }
        return true;
    }
    false
}