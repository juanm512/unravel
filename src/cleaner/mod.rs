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
    // 1. Calcular % de filas con errores
        let error_percentage = (report.rows_with_errors.len() as f64 / report.total_rows as f64) * 100.0;
        println!("Error percentage: {:.2}%", error_percentage);
    // 2. Si supera threshold → bail!
        if error_percentage.gt(&threshold) {
            println!("Error percentage exceeds threshold of {:.2}%. Aborting cleaning.", threshold);
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