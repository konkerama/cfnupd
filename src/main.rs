mod cfn;
mod helper;
use anyhow::{Context, Result};
use clap::Parser;
use std::{thread, time};

use crate::cfn::{describe_stack, get_params, get_template, init_client, update_stack};

use crate::helper::{cp_artifacts, get_editor, save_artifacts};
use rand::Rng;
use std::env;
use std::fs;
use std::process::Command;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
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

    // Whether to save modified artifacts to current directory
    #[structopt(short, long)]
    artifacts_to_current_dir: Option<bool>,
}

// static CHECK: Emoji<'_, '_> = Emoji("✅  ", "");

/// Retrieves the status of a CloudFormation stack in the Region.
/// # Arguments
///
/// * `-s STACK-NAME` - The name of the stack.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<()> {
    let Opt {
        region,
        stack_name,
        verbose,
        artifacts_to_current_dir,
    } = Opt::parse();

    if verbose {
        let subscriber = Registry::default()
            .with(LevelFilter::DEBUG)
            .with(tracing_subscriber::fmt::Layer::default().with_writer(std::io::stdout));

        tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

        tracing::debug!(
            "Arguments provided {{region: {:?}, stack_name: {}}}",
            region,
            stack_name
        )
    }

    let binding = env::temp_dir();
    let dir = binding.to_str().context("Issue finding tmp directory")?;

    let length = 10;
    let random_string: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    let directory = format!("{}/{}-{}", dir, stack_name, random_string);

    println!("Storing Artifacts in tmp Directory: {}", directory);
    fs::create_dir_all(&directory)?;

    let client = init_client(region).await;

    let _ = get_params(&client, &stack_name, &directory)
        .await
        .context("Unable to retrieve params")?;
    println!("✅ Successfully Received Cloudformation Parameters");

    // println!(
    //     "{} {}Resolving packages...",
    //     style("[1/4]").bold().dim(),
    //     CHECK
    // );

    let _ = get_template(&client, &stack_name, &directory)
        .await
        .context("Unable to retrieve template file")?;
    println!("✅ Successfully Received Cloudformation Template");

    let editor = get_editor().context("Unable to retrieve editor")?;

    Command::new(&editor)
        .arg(format!("{}/{}.yaml", directory, stack_name))
        .status()
        .context("Unable to edit template file")?;

    Command::new(&editor)
        .arg(format!("{}/parameters.json", directory))
        .status()
        .context("Unable to edit param file")?;

    let _ = update_stack(&client, &stack_name, &directory).await;

    loop {
        let binding = describe_stack(&client, &stack_name)
            .await
            .context("Unable to describe stack")?;
        let stack_status = binding.as_ref();
        if !stack_status.contains("IN_PROGRESS") {
            break;
        }
        thread::sleep(time::Duration::from_secs(5));
    }

    match artifacts_to_current_dir {
        Some(do_copy_artifacts) => {
            if do_copy_artifacts {
                cp_artifacts(&directory, &stack_name).context("Unable to save artifacts")?;
            }
        }
        None => {
            save_artifacts(directory, stack_name).context("Unable to save artifacts")?;
        }
    }

    Ok(())
}
