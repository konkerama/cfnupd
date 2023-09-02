use crate::helper::{json_to_param, ParamJson};
use anyhow::{Context, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation::types::Parameter;
use aws_sdk_cloudformation::types::StackStatus;
use aws_sdk_cloudformation::types::TemplateStage;
use aws_sdk_cloudformation::{config::Region, Client};
use console::style;
use std::fs;
use std::io::Write;
use std::{thread, time};

pub async fn init_client(region: Option<String>) -> Client {
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&shared_config)
}

pub async fn update_stack(
    client: &Client,
    name: &str,
    cfn_template_location: &String,
    cfn_parameters_location: &String,
) -> Result<()> {
    let contents =
        fs::read_to_string(cfn_template_location).expect("Something went wrong reading the file");
    tracing::debug!("update_stack::template: {:?}", contents);
    let params =
        fs::read_to_string(cfn_parameters_location).expect("Something went wrong reading the file");
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

    Ok(())
}

pub async fn get_template(
    client: &Client,
    name: &str,
    cfn_template_location: &String,
) -> Result<()> {
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

    fs::write(cfn_template_location, status).expect("Unable to write file");

    Ok(())
}

pub async fn get_params(
    client: &Client,
    name: &str,
    cfn_parameters_location: &String,
) -> Result<()> {
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

    let mut file = std::fs::File::create(cfn_parameters_location)?;
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

    print_status(status.as_ref());

    Ok(status)
}

fn print_status(status: &str) {
    if status.contains("ROLLBACK") {
        println!(
            "{} Stack status: {}",
            style("***").bold().dim(),
            style(status).red()
        );
    } else if status.contains("FAIL") {
        println!(
            "{} Stack status: {}",
            style("***").bold().dim(),
            style(status).red()
        );
    } else if status.contains("IN_PROGRESS") {
        println!(
            "{} Stack status: {}",
            style("***").bold().dim(),
            style(status).blue()
        );
    } else {
        println!(
            "{} Stack status: {}",
            style("***").bold().dim(),
            style(status).green()
        );
    }
}

pub async fn get_stack_feedback(client: &Client, stack_name: &str) -> Result<()> {
    loop {
        let binding = describe_stack(&client, &stack_name)
            .await
            .context("Unable to describe stack")?;
        let stack_status = binding.as_ref();
        if !stack_status.contains("IN_PROGRESS") {
            break;
        }
        thread::sleep(time::Duration::from_secs(3));
    }
    Ok(())
}
