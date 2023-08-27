use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation::types::Parameter;
use aws_sdk_cloudformation::types::builders::ParameterBuilder;
use aws_sdk_cloudformation::types::TemplateStage;
use aws_sdk_cloudformation::{config::Region, meta::PKG_VERSION, Client, Error};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::io;

#[derive(Serialize, Deserialize, Debug)]
struct ParamJson {
    pub parameter_key: String,
    pub parameter_value: String,
    pub use_previous_value: bool,
    pub resolved_value: String,
}

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The name of the AWS CloudFormation stack.
    #[structopt(short, long)]
    stack_name: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// Lists the status of a stack.
// snippet-start:[cloudformation.rust.describe-stack]
async fn describe_stack(client: &Client, name: &str) -> Result<(), Error> {
    // Return an error if stack_name does not exist
    let resp = client.describe_stacks().stack_name(name).send().await?;

    // println!("{:?}", resp);

    // Otherwise we get an array of stacks that match the stack_name.
    // The array should only have one item, so just access it via first().
    let status = resp
        .stacks()
        .unwrap_or_default()
        .first()
        .unwrap()
        .stack_status();

    println!("Stack status: {}", status.unwrap().as_ref());

    println!();

    Ok(())
}
// snippet-end:[cloudformation.rust.describe-stack]

// fn write_to_json(key: &str, value: &str, prev: bool, resolved: &str) {}

async fn get_params(client: &Client, name: &str) -> Result<(), Error> {
    // Return an error if stack_name does not exist
    let resp = client.describe_stacks().stack_name(name).send().await?;

    //    println!("{:?}", resp);

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
        // let serialized = serde_json::to_value(p).unwrap();
        paramlist.push(p);
    }
    // let j = serde_json::to_string(&paramlist).unwrap();
    // println!("{}",j);

    let params_ser = serde_json::to_string_pretty(&paramlist).unwrap();

    let mut file = std::fs::File::create("cfn/parms.json").unwrap();
    file.write_all(params_ser.as_bytes()).unwrap();
    // serde_json::to_writer_pretty(&mut file, &j).unwrap();

    println!("Stack status: {:?}", params);

    println!();

    Ok(())
}
// snippet-end:[cloudformation.rust.describe-stack]

fn json_to_param(s: String) -> Option<Vec<Parameter>>{
    let json_input: Vec<ParamJson> = serde_json::from_str(&s).unwrap();

    let mut p = Vec::new();
    for j in json_input{

        let param_builder = ParameterBuilder::default();

        let param = param_builder
                .set_parameter_key(Some(j.parameter_key))
                .set_parameter_value(Some(j.parameter_value))
                .set_use_previous_value(Some(j.use_previous_value))
                .set_resolved_value(Some(j.resolved_value))
                .build();
        p.push(param);
    }
    

    Some(p)
}



async fn update_stack(client: &Client, name: &str) -> Result<(), Error> {
    let contents =
        fs::read_to_string("cfn/asd.yaml").expect("Something went wrong reading the file");
    let params =
        fs::read_to_string("cfn/parms.json").expect("Something went wrong reading the file");
    
    // let input = match json_to_param(params){
    //     None => panic!(),
    //     Some(s) => s,
    // };
    let input: Option<Vec<Parameter>> = json_to_param(params);

    client
        .update_stack()
        .stack_name(name)
        .template_body(contents)
        .set_parameters(input)
        .send()
        .await?;

    println!("Stack created.");
    println!("Use describe-stacks with your stack name to see the status of your stack.");
    println!("You cannot use/deploy the stack until the status is 'CreateComplete'.");
    println!();

    Ok(())
}

// Lists the status of a stack.
// snippet-start:[cloudformation.rust.describe-stack]
async fn get_template(client: &Client, name: &str) -> Result<(), Error> {
    // Return an error if stack_name does not exist
    let resp = client
        .get_template()
        .stack_name(name)
        .template_stage(TemplateStage::Original)
        .send()
        .await?;

    // describe_stacks().stack_name(name).send().await?;

    //    println!("{:?}", resp);

    // Otherwise we get an array of stacks that match the stack_name.
    // The array should only have one item, so just access it via first().
    let status = resp.template_body().unwrap_or_default();
    //     .stacks()
    //     .unwrap_or_default()
    //     .first()
    //     .unwrap()
    //     .stack_status();

    println!("{:?}", &status);

    // let s : BTreeMap<String, f64> = serde_yaml::from_str(status).unwrap();
    fs::write("cfn/asd.yaml", status).expect("Unable to write file");

    // println!("{:?}", s);

    // println!("Stack status: {}", status.unwrap().as_ref());

    println!();

    Ok(())
}

/// Retrieves the status of a CloudFormation stack in the Region.
/// # Arguments
///
/// * `-s STACK-NAME` - The name of the stack.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        stack_name,
        verbose,
    } = Opt::parse();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("CloudFormation version: {}", PKG_VERSION);
        println!(
            "Region:                 {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("Stack:                  {}", &stack_name);
        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    get_params(&client, &stack_name).await;
    get_template(&client, &stack_name).await;

    // wait for modification of template
    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut user_input);
    update_stack(&client, &stack_name).await;


    Ok(())
}
