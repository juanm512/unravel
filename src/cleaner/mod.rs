use std::path::PathBuf;
use anyhow::{Ok, Result}; 

use crate::validator::ValidationReport;

pub fn clean(
    path: &PathBuf,
    headers: &[String],
    records: &[csv::StringRecord],
    report: &ValidationReport,
    threshold: f64,
) -> Result<PathBuf> {
    // 1. Si supera threshold → error
    if !should_clean(report, threshold) {
        return Err(anyhow::anyhow!("Error percentage exceeds threshold"));
    }
    // 3. Si no → escribir archivo limpio
    let mut wtr = csv::Writer::from_path(path.with_file_name(format!(
        "{}_cleaned.csv",
        path.file_stem().unwrap().to_string_lossy()
    )))?;
    
    // 4. Retornar path del archivo generado
    wtr.write_record(headers)?;
    for (i, record) in records.iter().enumerate() {
        // Los números de fila en el report son 1-indexed (fila 1 = header)
        // Los índices del array son 0-indexed, así que restamos 1 para comparar
        let row_number = i + 1;
        if !report.rows_with_errors.contains(&row_number) {
            wtr.write_record(record)?;
        }
    }
    wtr.flush()?;

    let cleaned_path = path.with_file_name(format!(
        "{}_cleaned.csv",
        path.file_stem().unwrap().to_string_lossy()
    ));

    Ok(cleaned_path)
}

fn should_clean(report: &ValidationReport, threshold: f64) -> bool {
    let error_percentage = (report.rows_with_errors.len() as f64 / report.total_rows as f64) * 100.0;
    error_percentage <= threshold
}

#[cfg(test)]                                                                                                                                                 
mod tests {                                                                                                                                                
    use std::collections::HashSet;
    use super::*;

    // threshold exactamente en el límite → pasa
    #[test]
    fn threshold_exactly_at_limit_passes() {
        // 1 error en 10 filas = 10.0% = exactamente el threshold → debe pasar
        let report = ValidationReport {
            total_rows: 10,
            rows_with_errors: HashSet::from([2]), // fila 2 tiene error
            errors: vec![],
            total_errors: 1, // 1 error
        };
        assert!(should_clean(&report, 10.0));
    }

    // threshold superado → falla
    #[test]
    fn threshold_exceeded_returns_error() {
        // 2 errores en 10 filas = 20.0% > 10.0% → debe fallar
        let report = ValidationReport {
            total_rows: 10,
            rows_with_errors: HashSet::from([2, 3]), // filas 2 y 3 tienen errores
            errors: vec![],
            total_errors: 2, // 2 errores
        };
        assert!(!should_clean(&report, 10.0));
    }

    #[test]

    fn threshold_zero_errors_passes() {
        // 0 errores en 10 filas = 0.0% ≤ 10.0% → debe pasar
        let report = ValidationReport {
            total_rows: 10,
            rows_with_errors: HashSet::from([]), // no hay errores
            errors: vec![],
            total_errors: 0, // 0 errores
        };
        assert!(should_clean(&report, 10.0));
    }
}