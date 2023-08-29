use crate::helper::{json_to_param, ParamJson};
use aws_sdk_cloudformation::types::Parameter;
use aws_sdk_cloudformation::types::StackStatus;
use aws_sdk_cloudformation::types::TemplateStage;
use aws_sdk_cloudformation::{config::Region, meta::PKG_VERSION, Client, Error};
use std::fs;
use std::io::Write;
use aws_config::meta::region::RegionProviderChain;



pub async fn init_client (region: Option<String>) -> Client{
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    // if verbose {
    //     println!("CloudFormation version: {}", PKG_VERSION);
    //     println!(
    //         "Region:                 {}",
    //         region_provider.region().await.unwrap().as_ref()
    //     );
    //     println!("Stack:                  {}", &stack_name);
    //     println!();
    // }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&shared_config)
}


pub async fn update_stack(client: &Client, name: &str) -> Result<(), Error> {
    let contents =
        fs::read_to_string("cfn/asd.yaml").expect("Something went wrong reading the file");
    let params =
        fs::read_to_string("cfn/parameters.json").expect("Something went wrong reading the file");

    let input: Option<Vec<Parameter>> = json_to_param(params);

    client
        .update_stack()
        .stack_name(name)
        .template_body(contents)
        .set_parameters(input)
        .send()
        .await?;

    println!("✅ Successfully Initiated Stack Update.");

    Ok(())
}

// Lists the status of a stack.
// snippet-start:[cloudformation.rust.describe-stack]
pub async fn get_template(client: &Client, name: &str) -> Result<(), Error> {
    // Return an error if stack_name does not exist
    let resp = client
        .get_template()
        .stack_name(name)
        .template_stage(TemplateStage::Original)
        .send()
        .await?;

    // Otherwise we get an array of stacks that match the stack_name.
    // The array should only have one item, so just access it via first().
    let status = resp.template_body().unwrap_or_default();

    fs::write("cfn/asd.yaml", status).expect("Unable to write file");

    Ok(())
}

pub async fn get_params(client: &Client, name: &str) -> Result<(), Error> {
    // Return an error if stack_name does not exist
    let resp = client.describe_stacks().stack_name(name).send().await?;

    // Otherwise we get an array of stacks that match the stack_name.
    // The array should only have one item, so just access it via first().
    let status = resp
        .stacks()
        .unwrap_or_default()
        .first()
        .unwrap()
        .parameters();

    let params: &[Parameter] = match status {
        None => panic!(),
        Some(i) => i,
    };
    let mut paramlist: Vec<ParamJson> = Vec::new();

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
        paramlist.push(p);
    }

    let params_ser = serde_json::to_string_pretty(&paramlist).unwrap();

    let mut file = std::fs::File::create("cfn/parameters.json").unwrap();
    file.write_all(params_ser.as_bytes()).unwrap();

    Ok(())
}

// Lists the status of a stack.
// snippet-start:[cloudformation.rust.describe-stack]
pub async fn describe_stack(client: &Client, name: &str) -> Result<StackStatus, Error> {
    // Return an error if stack_name does not exist
    let resp = client.describe_stacks().stack_name(name).send().await?;

    let status = resp
        .stacks()
        .unwrap_or_default()
        .first()
        .unwrap()
        .stack_status()
        .unwrap()
        .clone();

    println!("Stack status: {}", status.as_ref());

    Ok(status)
}
