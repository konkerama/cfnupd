use anyhow::{Context, Result};
use aws_sdk_cloudformation::types::builders::ParameterBuilder;
use aws_sdk_cloudformation::types::Parameter;
use config::Config;
use config::{Environment, File};
use serde::{Deserialize, Serialize};
use serde_json;
use std::env::var;
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamJson {
    pub parameter_key: String,
    pub parameter_value: String,
    pub use_previous_value: bool,
    pub resolved_value: String,
}

pub fn json_to_param(s: String) -> Result<Option<Vec<Parameter>>> {
    let json_input: Vec<ParamJson> = serde_json::from_str(&s)?;
    tracing::debug!("json_to_param::json_input: {:?}", json_input);

    let mut p = Vec::new();
    for j in json_input {
        let param_builder = ParameterBuilder::default();

        let param = param_builder
            .set_parameter_key(Some(j.parameter_key))
            .set_parameter_value(Some(j.parameter_value))
            .set_use_previous_value(Some(j.use_previous_value))
            .set_resolved_value(Some(j.resolved_value))
            .build();
        p.push(param);
    }
    tracing::debug!("json_to_param::parameters: {:?}", p);
    Ok(Some(p))
}

pub fn get_editor() -> Result<String> {
    let home_dir = dirs::home_dir().context("home dir not found")?;

    let config_dir = home_dir.join(".config");
    let config_path = config_dir.join("cfnupd.toml");
    tracing::debug!("get_editor::config_path: {:?}", config_path);

    // Check if the "EDITOR" environment variable is set
    if let Ok(editor) = var("EDITOR") {
        tracing::debug!("get_editor::env_var: {}", editor);
        return Ok(editor);
    }

    if config_path.exists() {
        let s = Config::builder()
            .add_source(File::with_name(
                config_path.to_str().context("Issue finding config file")?,
            ))
            .add_source(Environment::with_prefix("app"))
            .build()?;
        tracing::debug!("get_editor::found config file");
        return Ok(s.get::<String>("EDITOR")?);
    } else {
        // Create the config directory and file with default value ("nano")
        fs::create_dir_all(&config_dir)?;
        let mut file = fs::File::create(&config_path)?;
        file.write_all(b"EDITOR = \"nano\"")?;
        let default_value = "nano".to_string();
        tracing::debug!(
            "get_editor::creating config file with default value as {}",
            default_value
        );
        return Ok(default_value);
    }
}

pub fn cp_artifacts(directory: &String, stack_name: &String) -> Result<()> {
    fs::create_dir_all(stack_name)?;
    println! {"Copying artifacts ({}.yaml, parameter.json) under {}/", stack_name,stack_name}
    fs::copy(
        format!("{}/{}.yaml", directory, stack_name),
        format!("{}/{}.yaml", stack_name, stack_name),
    )?;
    fs::copy(
        format!("{}/parameters.json", directory),
        format!("{}/parameters.json", stack_name),
    )?;
    Ok(())
}

pub fn save_artifacts(directory: String, stack_name: String) -> Result<()> {
    loop {
        print!("Do you want to save the artifacts on the current directory (y,n) [Default 'n']: ");
        let _ = io::stdout().flush();
        let mut user_input = String::new();
        let stdin = io::stdin();
        let _ = stdin.read_line(&mut user_input);
        user_input = user_input.trim().to_string();

        match user_input.as_str() {
            "y" | "Y" | "yes" | "YES" | "Yes" => {
                let _ = cp_artifacts(&directory, &stack_name);
                break;
            }
            "n" | "N" | "No" | "NO" | "" => {
                println!("Artifacts are not saved in the current directory");
                break;
            }
            _ => println!("Invalid input, please try again"),
        }
    }
    Ok(())
}
