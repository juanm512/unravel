use std::collections::{HashMap, HashSet, hash_map::Entry};

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
    let columns_indexes: HashMap<&str, usize> = headers
      .iter()
      .enumerate()
      .map(|(i, h)| (h.as_str(), i))
      .collect();
    let pre_parsed: HashMap<&str, (Option<NaiveDate>, Option<NaiveDate>)> = rules.columns
      .iter()
      .filter_map(|(name, config)| {
          if let ColumnRule::Date { before, after } = &config.rule {
              let b = before.as_deref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
              let a = after.as_deref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
              Some((name.as_str(), (b, a)))
          } else { None }
      })
      .collect();

    let mut unique_values: HashMap<&str, HashMap<String, usize>> = HashMap::new();

    for (i, record) in records.iter().enumerate() {
        // iteramos sobre cada regla y validamos la columna correspondiente
        for rule in &rules.columns {
            let column_name = rule.0;
            let column_config = &rule.1;

           if let Some(&column_index) = columns_indexes.get(column_name.as_str()) {
                let value = record.get(column_index).unwrap_or("");
                let is_empty = value.trim().is_empty();

                if column_config.required && is_empty {
                    report.add_error(i + 1, ValidationError {
                        column: column_name.clone(),
                        row: i + 1,
                        message: format!("Value in column '{}' is required", column_name),
                        value: Some(value.to_string()),
                    });
                    continue; // Si es requerido y está vacío, no hace falta validar el formato
                }
                if is_empty {
                    continue; // no-required y vacío → skip tipo
                }

                if column_config.unique {
                    match unique_values.entry(column_name).or_default().entry(value.to_string()) {
                        Entry::Occupied(e) => {
                            let existing_row = *e.get();
                            report.add_error(i + 1, ValidationError {
                                column: column_name.clone(),
                                row: i + 1,
                                message: format!("Value '{}' in column '{}' is duplicated (also found in row {})", value, column_name, existing_row),
                                value: Some(value.to_string()),
                            });
                        },
                        Entry::Vacant(e)   => { e.insert(i + 1); }
                    }
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
                    ColumnRule::Date { before: _, after: _ } => {
                        let (before_date, after_date) = pre_parsed.get(column_name.as_str()).unwrap_or(&(None, None));
                        if !validate_date(value, *before_date, *after_date) {
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

    report.set_total_rows(records.len());
    report
}


pub fn validate_email(email: &str, email_regex: &Regex) -> bool {
    email_regex.is_match(email)
}

pub fn validate_pattern(value: &str, pattern: &Regex) -> bool {
    pattern.is_match(value)
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

pub fn validate_date(value: &str, before: Option<NaiveDate>, after: Option<NaiveDate>) -> bool {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(before_date) = before {
            if date >= before_date {
                return false;
            }
        }
        if let Some(after_date) = after {
            if date <= after_date {
                return false;
            }
        }
        return true;
    }
    false
}