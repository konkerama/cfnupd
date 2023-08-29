mod cfn;
mod helper;

use aws_sdk_cloudformation::Error;
use clap::Parser;
use std::{thread, time};

use crate::cfn::{describe_stack, get_params, get_template, update_stack, init_client};
use crate::helper::get_editor;
use std::fs;
use std::{
    // env::{temp_dir, var},
    process::Command,
};

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

    fs::create_dir_all("cfn").unwrap();

    let Opt {
        region,
        stack_name,
        verbose,
    } = Opt::parse();

    let client  = init_client(region).await;

    let _ = get_params(&client, &stack_name).await;
    println!("✅ Successfully Received Cloudformation Template");

    let _ = get_template(&client, &stack_name).await;
    println!("✅ Successfully Received Cloudformation Parameters");

    let editor = get_editor();

    Command::new(&editor)
        .arg("cfn/asd.yaml")
        .status()
        .expect("Something went wrong");

    Command::new(&editor)
        .arg("cfn/parameters.json")
        .status()
        .expect("Something went wrong");

    let _ = update_stack(&client, &stack_name).await;

    loop {
        let binding = describe_stack(&client, &stack_name).await.unwrap();
        let stack_status = binding.as_ref();
        if !stack_status.contains("IN_PROGRESS") {
            break;
        }
        thread::sleep(time::Duration::from_secs(5));
    }

    Ok(())
}
