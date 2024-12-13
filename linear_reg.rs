use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use ndarray::{Array2, Array1};

pub fn one_hot_encode(
    input_path: &str,
    output_path: &str,
    reference_group: &str,
    target_columns: &[&str], 
) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    let header = lines.next().ok_or("Missing header line")??;
    let columns: Vec<&str> = header.split(',').collect();

    let target_indices: HashMap<&str, usize> = target_columns
        .iter()
        .filter_map(|&col| {
            columns
                .iter()
                .position(|&header_col| header_col == col)
                .map(|index| (col, index))
        })
        .collect();

    if target_indices.is_empty() {
        return Err("No target columns found in the dataset header".into());
    }

    let mut unique_values: HashMap<&str, Vec<String>> = HashMap::new();
    let mut data = Vec::new();

    for line in lines {
        let line = line?;
        let row: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
        data.push(row.clone());

        for (&col, &index) in &target_indices {
            if let Some(value) = row.get(index) {
                let unique_list = unique_values.entry(col).or_insert_with(Vec::new);
                if !unique_list.contains(value) && value != reference_group {
                    unique_list.push(value.clone());
                }
            }
        }
    }

    let mut output_file = File::create(output_path)?;

    let mut new_header: Vec<String> = Vec::new();
    for &col in target_columns {
        new_header.push(col.to_string()); 
        if let Some(unique_list) = unique_values.get(col) {
            for value in unique_list {
                let new_column_name = format!("{}_is_{}", col, value);
                new_header.push(new_column_name);
            }
        }
    }

    output_file.write_all(new_header.join(",").as_bytes())?;
    output_file.write_all(b"\n")?;

    for row in data {
        let mut new_row = Vec::new();

        for &col in target_columns {
            if let Some(&index) = target_indices.get(col) {
                if let Some(value) = row.get(index) {
                    new_row.push(value.clone());
                } else {
                    new_row.push("".to_string());
                }
            }
        }

        for &col in target_columns {
            if let Some(unique_list) = unique_values.get(col) {
                if let Some(&index) = target_indices.get(col) {
                    for value in unique_list {
                        let is_match = if let Some(row_value) = row.get(index) {
                            row_value == value
                        } else {
                            false
                        };
                        new_row.push(if is_match { "1" } else { "0" }.to_string());
                    }
                }
            }
        }

        output_file.write_all(new_row.join(",").as_bytes())?;
        output_file.write_all(b"\n")?;
    }

    Ok(())
}


pub fn matrix(
    input_path: &str,
    reference_group: &str,
    target_column: &str, 
) -> Result<(Array2<f64>, Array1<f64>), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    let header = lines.next().ok_or("Missing header line")??;
    let columns: Vec<&str> = header.split(',').collect();

    let target_index = columns
        .iter()
        .position(|&col| col == target_column)
        .ok_or("Target column not found in the header")?;

    let mut unique_values: Vec<String> = Vec::new();
    let mut rows = Vec::new();

    for line in lines {
        let line = line?;
        let row: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
        if let Some(value) = row.get(target_index) {
            if !unique_values.contains(value) {
                unique_values.push(value.clone());
            }
        }
        rows.push(row);
    }

    unique_values.retain(|val| val != reference_group);

    let num_rows = rows.len();
    let num_predictors = unique_values.len();
    let mut x_data = Vec::new();
    let mut y_data = Vec::new();

    for row in rows {
        if let Some(value) = row.get(target_index) {
            y_data.push(if value == reference_group { 1.0 } else { 0.0 });
        }

        let mut predictors = Vec::new();
        for unique_value in &unique_values {
            if let Some(value) = row.get(target_index) {
                predictors.push(if value == unique_value { 1.0 } else { 0.0 });
            }
        }
        x_data.extend(predictors);
    }

    let x_matrix = Array2::from_shape_vec((num_rows, num_predictors), x_data)?;
    let y_vector = Array1::from_vec(y_data);

    Ok((x_matrix, y_vector))
}
