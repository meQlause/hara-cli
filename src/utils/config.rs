use std::env;
use std::io::{self, Write, BufRead};
use eyre::{Result, eyre};

/// Retrieves an environment variable or prompts the user to provide it interactively.
pub fn get_or_prompt(key: &str, description: &str, example: &str) -> Result<String> {
    if let Ok(val) = env::var(key) {
        let trimmed = val.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    tracing::warn!("Missing configuration: {}", key);
    tracing::info!("Tutorial: You can set this in your profile or .env file:");
    tracing::info!("   export {}={}", key, example);
    
    print!("\nWould you like to input the {} now? (y/N): ", description);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    if choice == "y" || choice == "yes" {
        print!("Enter {}: ", description);
        io::stdout().flush().ok();
        let mut value = String::new();
        io::stdin().lock().read_line(&mut value)?;
        let value = value.trim().to_string();

        if !value.is_empty() {
            update_env(key, &value).ok();
            return Ok(value);
        }
    }

    Err(eyre!(
        "Configuration Error: {} is not set.\nRun: export {}={}",
        key,
        key,
        example
    ))
}

/// Forcefully prompts the user for a value, ignoring environment variables.
pub fn prompt(description: &str, example: &str) -> Result<String> {
    print!("Enter {} (e.g. {}): ", description, example);
    io::stdout().flush().ok();

    let mut value = String::new();
    io::stdin().lock().read_line(&mut value)?;
    let value = value.trim().to_string();

    if value.is_empty() {
        return Err(eyre!("Input for {} cannot be empty", description));
    }

    Ok(value)
}

/// Updates or appends a key-value pair in the .env file.
pub fn update_env(key: &str, value: &str) -> Result<()> {
    let path = ".env";
    let mut lines = Vec::new();
    let mut found = false;

    if std::path::Path::new(path).exists() {
        let file = std::fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with(key) && line.contains('=') {
                lines.push(format!("{}={}", key, value));
                found = true;
            } else {
                lines.push(line);
            }
        }
    }

    if !found {
        lines.push(format!("{}={}", key, value));
    }

    let mut file = std::fs::File::create(path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }
    
    tracing::info!("Updated {} in .env", key);
    Ok(())
}
