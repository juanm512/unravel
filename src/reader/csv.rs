use std::path::PathBuf;

use anyhow::Result;

pub fn load(path: &PathBuf) -> Result<(Vec<String>, Vec<csv::StringRecord>)> {
    // check if file exists and extension is .csv
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {:?}", path));
    }
    if path.extension().unwrap_or_default() != "csv" {
        return Err(anyhow::anyhow!("Invalid file extension: {:?}", path));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let headers = reader.headers()?.iter().map(|s| s.to_string()).collect();
    let records = reader.records().collect::<Result<_, csv::Error>>()?;

    Ok((headers, records))
}