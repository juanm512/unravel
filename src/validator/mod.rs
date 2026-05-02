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

    for column_name in rules.columns.keys() {
        if !columns_indexes.contains_key(column_name.as_str()) {
            eprintln!("[WARNING] Column '{}' defined in rules but not found in CSV", column_name);
        }
    }

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
            if date > before_date {
                return false;
            }
        }
        if let Some(after_date) = after {
            if date < after_date {
                return false;
            }
        }
        return true;
    }
    false
}



#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_email ---
    #[test]
    fn email_valid_format_is_valid() {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        assert!(validate_email("test@example.com", &email_regex));
    }
    
    #[test]
    fn email_invalid_format_is_invalid() {
      let r = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
      assert!(!validate_email("not-an-email", &r));
      assert!(!validate_email("missing@dot", &r));
  }

    // --- validate_pattern ---
    #[test]
    fn pattern_valid_format_is_valid() {
        let pattern_regex = Regex::new(r"^[A-Z]{3}-\d{4}$").unwrap();
        assert!(validate_pattern("ABC-1234", &pattern_regex));
        assert!(!validate_pattern("abc-1234", &pattern_regex));
    }

    // --- validate_float ---
    #[test]
    fn float_min_boundary_is_valid() {
        assert!(validate_float("1.0", Some(1.0), Some(10.0)));
    }

    #[test]
    fn float_max_boundary_is_valid() {
        assert!(validate_float("10.0", Some(1.0), Some(10.0)));
    }

    #[test]
    fn float_below_min_is_invalid() {
        assert!(!validate_float("0.9", Some(1.0), Some(10.0)));
    }

    #[test]
    fn float_above_max_is_invalid() {
        assert!(!validate_float("10.1", Some(1.0), Some(10.0)));
    }

    // --- validate_integer ---
    #[test]
    fn integer_min_boundary_is_valid() {
        assert!(validate_integer("1", Some(1), Some(10)));
    }

    #[test]
    fn integer_max_boundary_is_valid() {
        assert!(validate_integer("10", Some(1), Some(10)));
    }

    #[test]
    fn integer_below_min_is_invalid() {
        assert!(!validate_integer("0", Some(1), Some(10)));
    }

    #[test]
    fn integer_above_max_is_invalid() {
        assert!(!validate_integer("11", Some(1), Some(10)));
    }

    #[test]
    fn integer_no_bounds_is_valid() {
        assert!(validate_integer("99999", None, None));
    }

    // --- validate_date ---
    #[test]
    fn date_exactly_on_before_is_valid() {
        assert!(validate_date("2025-01-01", Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()), None));
    }

    #[test]
    fn date_after_before_is_invalid() {
        assert!(!validate_date("2025-01-02", Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()), None));
    }

    #[test]
    fn date_exactly_on_after_is_valid() {
        assert!(validate_date("2020-01-01", None, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())));
    }

    #[test]
    fn date_before_after_is_invalid() {
        assert!(!validate_date("2019-12-31", None, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())));
    }

    #[test]
    fn date_invalid_format_is_invalid() {
        assert!(!validate_date("31-01-2025", None, None));
    }


    // helpers
    use crate::reader::yaml::Rules;
    use csv::StringRecord;
    use serde_yaml;

    fn rules_from_yaml(yaml: &str) -> Rules {
        serde_yaml::from_str(yaml).unwrap()
    }

    fn record(values: &[&str]) -> StringRecord {
        StringRecord::from(values.to_vec())
    }

    // --- Tests de validate()

    // --- required ---
    #[test]
    fn required_empty_cell_is_error() {
        let rules = rules_from_yaml("columns:\n  name:\n    rule:\n      type: text\n    required: true");
        let headers = vec!["name".to_string()];
        let records = vec![record(&[""])];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 1);
        assert!(report.rows_with_errors.contains(&1));
    }

    #[test]
    fn required_spaces_only_is_error() {
        let rules = rules_from_yaml("columns:\n  name:\n    rule:\n      type: text\n    required: true");
        let headers = vec!["name".to_string()];
        let records = vec![record(&["   "])];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 1);
    }

    #[test]
    fn required_with_value_is_valid() {
        let rules = rules_from_yaml("columns:\n  name:\n    rule:\n      type: text\n    required: true");
        let headers = vec!["name".to_string()];
        let records = vec![record(&["John"])];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 0);
    }

    // --- unique ---
    #[test]
    fn unique_duplicate_is_error() {
        let rules = rules_from_yaml("columns:\n  email:\n    rule:\n      type: text\n    unique: true");
        let headers = vec!["email".to_string()];
        let records = vec![
            record(&["a@b.com"]),
            record(&["c@d.com"]),
            record(&["c@d.com"]),  // duplicado de fila 2
        ];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 1);
        assert!(report.rows_with_errors.contains(&3));
    }

    #[test]
    fn unique_no_duplicates_is_valid() {
        let rules = rules_from_yaml("columns:\n  email:\n    rule:\n      type: text\n    unique: true");
        let headers = vec!["email".to_string()];
        let records = vec![record(&["a@b.com"]), record(&["c@d.com"])];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 0);
    }

    // --- columna en rules que no existe en CSV ---
    #[test]
    fn missing_column_in_csv_produces_no_errors() {
        let rules = rules_from_yaml("columns:\n  nonexistent:\n    rule:\n      type: text\n    required: true");
        let headers = vec!["other".to_string()];
        let records = vec![record(&["value"])];
        let report = validate(&headers, &records, &rules);
        // warning en stderr, pero ningún error en el reporte
        assert_eq!(report.total_errors, 0);
    }

    // --- columna en CSV que no está en rules ---
    #[test]
    fn extra_column_in_csv_is_ignored() {
        let rules = rules_from_yaml("columns:\n  name:\n    rule:\n      type: text");
        let headers = vec!["name".to_string(), "extra".to_string()];
        let records = vec![record(&["John", "ignored"])];
        let report = validate(&headers, &records, &rules);
        assert_eq!(report.total_errors, 0);
    }
}