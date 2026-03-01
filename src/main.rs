use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get input file path from user
    print!("Enter the path to the input JSON file: ");
    io::stdout().flush()?;
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();

    // 2. Get output file path from user
    print!("Enter the path for the output CSV file: ");
    io::stdout().flush()?;
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();

    // 3. Read and parse the JSON file
    let mut file = File::open(Path::new(input_path))?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;

    let root: serde_json::Value = serde_json::from_str(&json_data)?;

    // 4. Extract data from main object > "items" array > "items" array
    let mut data_records = Vec::new();

    if let Some(outer_items) = root.get("items").and_then(|v| v.as_array()) {
        for outer_item in outer_items {
            if let Some(inner_items) = outer_item.get("items").and_then(|v| v.as_array()) {
                for inner_item in inner_items {
                    if let Some(obj) = inner_item.as_object() {
                        data_records.push(obj);
                    }
                }
            }
        }
    }

    if data_records.is_empty() {
        println!("No data found matching the specified structure (main object > 'items' > 'items').");
        return Ok(());
    }

    // 5. Gather all unique keys across all records to use as CSV headers
    let mut keys = HashSet::new();
    for record in &data_records {
        for key in record.keys() {
            keys.insert(key.clone());
        }
    }
    
    // Sort keys alphabetically so columns are predictable
    let mut headers: Vec<String> = keys.into_iter().collect();
    headers.sort();

    // 6. Set up the CSV Writer
    let mut wtr = csv::Writer::from_path(output_path)?;

    // Write headers
    wtr.write_record(&headers)?;

    // 7. Write data rows
    for record in data_records {
        let mut row = Vec::new();
        for header in &headers {
            let value = match record.get(header) {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                Some(serde_json::Value::Null) => String::new(),
                Some(other) => other.to_string(), // Fallback for arrays/objects converted to strings
                None => String::new(), // Handle missing fields
            };
            row.push(value);
        }
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    println!("Successfully converted {} to {}", input_path, output_path);

    Ok(())
}