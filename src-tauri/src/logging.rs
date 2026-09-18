use std::collections::HashMap;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use crate::Error;

// -----------------------------
// HELPER FUNCTIONS
// -----------------------------

pub fn read_workout_logs<R: Runtime>(app: &AppHandle<R>, file_name: &str) -> Result<HashMap<String, Value>, Error> {
    let mut path = app.path().app_data_dir()
        .map_err(|e| Error { message: e.to_string() })?;
    
    path.push(file_name);

    // 1. If the file doesn't exist yet, return an empty HashMap
    if !path.exists() {
        return Ok(HashMap::new());
    }

    // 2. Read the file contents into a string
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error { message: e.to_string() })?;

    // 3. If the file content is empty/whitespace, return an empty HashMap
    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    // 4. Parse the JSON string into the HashMap. 
    // unwrap_or_default() ensures fallback to empty instead of crashing on bad JSON.
    let logs: HashMap<String, Value> = serde_json::from_str(&content)
        .unwrap_or_default();

    Ok(logs)
}

pub fn save_workout_logs<R: Runtime>(app: &AppHandle<R>, file_name: &str, logs: &HashMap<String, Value>) -> Result<(), Error> {
    let mut path = app.path().app_data_dir()
        .map_err(|e| Error { message: e.to_string() })?;
    
    // Ensure the app data directory exists before saving
    std::fs::create_dir_all(&path)
        .map_err(|e| Error { message: e.to_string() })?;
    
    path.push(file_name);

    let json_data = serde_json::to_string_pretty(logs)
        .map_err(|e| Error { message: e.to_string() })?;
        
    std::fs::write(path, json_data)
        .map_err(|e| Error { message: e.to_string() })?;

    Ok(())
}

// -----------------------------
// TAURI COMMANDS
// -----------------------------

#[tauri::command]
pub async fn log_workout<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    params: Value,
    date: String
) -> Result<(), Error> {
    let mut logs = read_workout_logs(&app, &file_name)?;
    let date_key = date.trim().to_string();

    // 1. Ensure an array exists at this key
    let entry = logs.entry(date_key).or_insert_with(|| Value::Array(Vec::new()));

    // 2. Safely push to the array
    if let Some(vec) = entry.as_array_mut() {
        vec.push(params);
    } else {
        // Fallback safety: if legacy non-array data was stored there, convert it into an array
        let old_val = entry.clone();
        *entry = Value::Array(vec![old_val, params]);
    }

    // 3. Save the updated logs
    save_workout_logs(&app, &file_name, &logs)?;

    Ok(())
}

#[tauri::command]
pub async fn delete_workout_log<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    date: String,
    index: usize, // 0-based index of the workout to remove
) -> Result<(), Error> {
    let mut logs = read_workout_logs(&app, &file_name)?;
    let date_key = date.trim();

    let mut should_remove_key = false;

    // 1. Check if the key exists and grab a mutable reference to the array
    if let Some(value) = logs.get_mut(date_key) {
        if let Some(vec) = value.as_array_mut() {
            
            // 2. Only remove if index is within bounds (prevents panics from bad JS inputs)
            if index < vec.len() {
                vec.remove(index);
            }

            // 3. Clean up the key entirely if no workouts remain for that day
            if vec.is_empty() {
                should_remove_key = true;
            }
        }
    }

    // 4. Remove the empty array key so our JSON file stays clean
    if should_remove_key {
        logs.remove(date_key);
    }

    // 5. Save the updated logs
    save_workout_logs(&app, &file_name, &logs)?;

    Ok(())
}