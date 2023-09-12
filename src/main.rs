mod cfn;
mod helper;
use anyhow::{Context, Result};
use clap::Parser;

use crate::cfn::{get_params, get_stack_feedback, get_template, init_client, update_stack};
use crate::helper::{modify_artifacts, save_artifacts_if_needed, CfnLocations};
use console::{style, Emoji};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
#[derive(Debug, Parser)]
struct Opt {
    /// The name of the stack you want to update.
    #[structopt(short, long)]
    stack_name: String,

    /// The region in which the AWS Cloudformation stack you want to update exists. 
    /// If not provided value is retrieved from the AWS Config. (eg eu-west-1)
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether or not to print verbose logs on the stdout. 
    /// To be used only for debug purposes.
    #[structopt(short, long)]
    editor: Option<String>,

    /// Provide the necessary Cloudformation capabilities required for the update to be performed (CapabilityIam/CapabilityNamedIam/CapabilityAutoExpand). 
    /// If not provided and the update requires any of this capabilities then the update will fail.
    #[structopt(short, long)]
    capabilities: bool,

    /// Whether or not to save the updated artifacts to the current directory. 
    /// If not specified the used gets a prompt after the modification and the update occurs.
    #[structopt(short, long)]
    artifacts_to_current_dir: Option<bool>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

static CHECK: Emoji<'_, '_> = Emoji("✅  ", "");

/// Retrieves the status of a CloudFormation stack in the Region.
/// # Arguments
///
/// * `-s STACK-NAME` - The name of the stack.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **eu-west-1**.
/// * `[-a]` - Whether or not to save artifacts to current directory
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<()> {
    let Opt {
        region,
        stack_name,
        verbose,
        artifacts_to_current_dir,
        capabilities,
        editor,
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

    let cfn_locations =
        CfnLocations::init(&stack_name).context("Unable to identify cfn file locations")?;

    let client = init_client(region).await;

    let _ = get_template(
        &client,
        &stack_name,
        &cfn_locations.tmp_cfn_template_location,
    )
    .await
    .context("Unable to retrieve template file")?;

    println!(
        "{} {}Successfully Received Cloudformation Template",
        style("[2/5]").bold().dim(),
        CHECK
    );

    let _ = get_params(
        &client,
        &stack_name,
        &cfn_locations.tmp_cfn_parameters_location,
    )
    .await
    .context("Unable to retrieve params")?;

    println!(
        "{} {}Successfully Received Cloudformation Parameters",
        style("[3/5]").bold().dim(),
        CHECK
    );

    let _ = modify_artifacts(
        editor,
        &cfn_locations.tmp_cfn_template_location,
        &cfn_locations.tmp_cfn_parameters_location,
    )
    .context("Unable to modify artifacts")?;
    println!(
        "{} {}Successfully Modified Artifacts",
        style("[4/5]").bold().dim(),
        CHECK
    );

    let _ = update_stack(
        &client,
        &stack_name,
        capabilities,
        &cfn_locations.tmp_cfn_template_location,
        &cfn_locations.tmp_cfn_parameters_location,
    )
    .await
    .context("Issue updating Stack")?;

    println!(
        "{} {}Successfully Initiated Stack Update",
        style("[5/5]").bold().dim(),
        CHECK
    );

    let _ = get_stack_feedback(&client, &stack_name)
        .await
        .context("Issue receiving stack feedback")?;

    let _ = save_artifacts_if_needed(artifacts_to_current_dir, cfn_locations)
        .context("Issue saving artifacts to current directory")?;

    Ok(())
}
