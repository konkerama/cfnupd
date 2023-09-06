use anyhow::{Context, Result};
use aws_sdk_cloudformation::types::builders::ParameterBuilder;
use aws_sdk_cloudformation::types::Parameter;
use config::Config;
use config::{Environment, File};
use console::{style, Emoji};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::env::var;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

static DIRECTORY: Emoji<'_, '_> = Emoji("📁  ", "");

static CONFIG_DIRECTORY_NAME: &str = "cfnupd";
static CONFIG_FILENAME: &str = "config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamJson {
    pub parameter_key: String,
    pub parameter_value: String,
    pub use_previous_value: bool,
    pub resolved_value: String,
}

pub struct CfnLocations {
    pub tmp_directory: String,
    pub tmp_cfn_template_location: String,
    pub tmp_cfn_parameters_location: String,
    pub target_directory: String,
    pub target_cfn_template_location: String,
    pub target_cfn_parameters_location: String,
}

impl CfnLocations {
    pub fn init(stack_name: &String) -> Result<Self> {
        let tmp_directory =
            identify_tmp_directory(&stack_name).context("Issue setting up tmp directory")?;

        let tmp_cfn_template_location = format!("{}/{}.yaml", tmp_directory, stack_name);
        let tmp_cfn_parameters_location = format!("{}/parameters.json", tmp_directory);

        let target_cfn_template_location = format!("{}/{}.yaml", stack_name, stack_name);
        let target_cfn_parameters_location = format!("{}/parameters.json", stack_name);
        let target_directory = format!("{}", stack_name);

        Ok(CfnLocations {
            tmp_directory,
            tmp_cfn_template_location,
            tmp_cfn_parameters_location,
            target_directory,
            target_cfn_template_location,
            target_cfn_parameters_location,
        })
    }
}

// Identifying location of tmp directory based on the OS and creates a subdirectory
// to store the to be updated artifacts
pub fn identify_tmp_directory(stack_name: &String) -> Result<String> {
    let binding = env::temp_dir();
    let dir = binding.to_str().context("Issue finding tmp directory")?;

    let length = 10;
    let random_string: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    let directory = format!("{}/{}-{}", dir, stack_name, random_string);

    println!(
        "{} {}Storing Artifacts in tmp Directory: {}",
        style("[1/5]").bold().dim(),
        DIRECTORY,
        directory
    );

    fs::create_dir_all(&directory)?;
    Ok(directory)
}

// Converts the json input to the CFN params struct
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

fn get_config_file_location() -> Result<(PathBuf, PathBuf)> {
    let config_prefix = dirs::config_dir().context("Unable to retrieve config file location")?;
    let config_directory_name = config_prefix.join(CONFIG_DIRECTORY_NAME);
    let config_file_path = config_directory_name.join(CONFIG_FILENAME);
    tracing::debug!("get_editor::config_path: {:?}", config_file_path);

    Ok((config_directory_name, config_file_path))
}

// Get Editor for the current system.
// More info on the decision logic can be found on the documentation
pub fn get_editor() -> Result<String> {
    let (config_directory_name, config_path) = get_config_file_location()?;

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
        fs::create_dir_all(&config_directory_name)?;
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

// Copying artifacts from the tmp directory to the "current" directory for future use
pub fn cp_artifacts(cfn_locations: &CfnLocations) -> Result<()> {
    fs::create_dir_all(&cfn_locations.target_directory)?;
    println! {"Copying cfn artifacts to {}/", cfn_locations.target_directory}
    fs::copy(
        &cfn_locations.tmp_cfn_template_location,
        &cfn_locations.target_cfn_template_location,
    )?;
    fs::copy(
        &cfn_locations.tmp_cfn_parameters_location,
        &cfn_locations.target_cfn_parameters_location,
    )?;
    Ok(())
}

// Identifies whether of not there is a need for saving artifacts on the tmp directory to the
// "current" directory based on the user's input
pub fn save_artifacts_if_needed(
    artifacts_to_current_dir: Option<bool>,
    cfn_locations: CfnLocations,
) -> Result<()> {
    match artifacts_to_current_dir {
        Some(do_copy_artifacts) => {
            if do_copy_artifacts {
                let _ = cp_artifacts(&cfn_locations).context("Unable to save artifacts")?;
            }
        }
        None => {
            loop {
                print!("Do you want to save the artifacts on the current directory (y,n) [Default 'n']: ");
                let _ = io::stdout().flush();
                let mut user_input = String::new();
                let stdin = io::stdin();
                let _ = stdin.read_line(&mut user_input);
                user_input = user_input.trim().to_string();

                match user_input.as_str() {
                    "y" | "Y" | "yes" | "YES" | "Yes" => {
                        let _ = cp_artifacts(&cfn_locations).context("Unable to save artifacts")?;
                        break;
                    }
                    "n" | "N" | "No" | "NO" | "" => {
                        println!("Artifacts are not saved in the current directory");
                        break;
                    }
                    _ => println!("{}", style("Invalid input, please try again").bold().dim()),
                }
            }
        }
    }

    Ok(())
}

// Modify downloaded artifacts using the configured file editor
pub fn modify_artifacts(
    cfn_template_location: &String,
    cfn_parameters_location: &String,
) -> Result<()> {
    let editor = get_editor().context("Unable to retrieve editor")?;

    Command::new(&editor)
        .arg(cfn_template_location)
        .status()
        .context("Unable to edit template file")?;

    Command::new(&editor)
        .arg(cfn_parameters_location)
        .status()
        .context("Unable to edit param file")?;
    Ok(())
}
