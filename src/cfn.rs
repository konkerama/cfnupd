use crate::helper::{json_to_param, ParamJson};
use anyhow::{Context, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation::types::Parameter;
use aws_sdk_cloudformation::types::StackStatus;
use aws_sdk_cloudformation::types::TemplateStage;
use aws_sdk_cloudformation::{config::Region, Client};
use std::fs;
use std::io::Write;

pub async fn init_client(region: Option<String>) -> Client {
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&shared_config)
}

pub async fn update_stack(client: &Client, name: &str, directory: &String) -> Result<()> {
    let contents = fs::read_to_string(format!("{}/{}.yaml", directory, name))
        .expect("Something went wrong reading the file");
    tracing::debug!("update_stack::template: {:?}", contents);
    let params = fs::read_to_string(format!("{}/parameters.json", directory))
        .expect("Something went wrong reading the file");
    tracing::debug!("update_stack::parameters: {:?}", params);

    let input: Option<Vec<Parameter>> = json_to_param(params)?;

    let resp = client
        .update_stack()
        .stack_name(name)
        .template_body(contents)
        .set_parameters(input)
        .send()
        .await?;

    tracing::debug!("update_stack::update_stack_response: {:?}", resp);

    println!("✅ Successfully Initiated Stack Update.");

    Ok(())
}

pub async fn get_template(client: &Client, name: &str, directory: &String) -> Result<()> {
    // Return an error if stack_name does not exist
    let resp = client
        .get_template()
        .stack_name(name)
        .template_stage(TemplateStage::Original)
        .send()
        .await?;

    tracing::debug!("get_template::get_template_response: {:?}", resp);

    let status = resp
        .template_body()
        .context("unable to retrieve template body")?;
    tracing::debug!("get_template::get_template_response_body: {:?}", status);

    let file_path = format!("{}/{}.yaml", directory, name);

    fs::write(file_path, status).expect("Unable to write file");

    Ok(())
}

pub async fn get_params(client: &Client, name: &str, directory: &String) -> Result<()> {
    // Return an error if stack_name does not exist
    let resp = client.describe_stacks().stack_name(name).send().await?;

    tracing::debug!("get_params::describe_stacks_response: {:?}", resp);

    let status = resp
        .stacks()
        .context("Error extracting parameter information")?
        .first()
        .context("Error extracting parameter information")?
        .parameters();

    tracing::debug!("get_params::stack_parameters: {:?}", status);

    let params: &[Parameter] = match status {
        None => panic!(),
        Some(i) => i,
    };
    let mut parameter_list: Vec<ParamJson> = Vec::new();

    for param in params {
        let key = match param.parameter_key() {
            None => "N/A".to_string(),
            Some(s) => s.to_string(),
        };
        let value = match param.parameter_value() {
            None => "N/A".to_string(),
            Some(s) => s.to_string(),
        };
        let prev = match param.use_previous_value() {
            None => false,
            Some(s) => s,
        };
        let resolved = match param.resolved_value() {
            None => "N/A".to_string(),
            Some(s) => s.to_string(),
        };
        let p = ParamJson {
            parameter_key: key,
            parameter_value: value,
            use_previous_value: prev,
            resolved_value: resolved,
        };
        parameter_list.push(p);
    }

    tracing::debug!("get_params::stack_parameters_list: {:?}", parameter_list);

    let params_ser = serde_json::to_string_pretty(&parameter_list)?;

    let file_path = format!("{}/parameters.json", directory);

    let mut file = std::fs::File::create(file_path)?;
    file.write_all(params_ser.as_bytes())?;

    Ok(())
}

pub async fn describe_stack(client: &Client, name: &str) -> Result<StackStatus> {
    let resp = client.describe_stacks().stack_name(name).send().await?;

    tracing::debug!("describe_stack::describe_stack_response: {:?}", resp);

    let status = resp
        .stacks()
        .context("Error extracting stack information")?
        .first()
        .context("Error extracting stack information")?
        .stack_status()
        .context("Error extracting stack information")?
        .clone();

    println!("Stack status: {}", status.as_ref());

    Ok(status)
}
