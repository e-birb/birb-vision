use std::path::PathBuf;

use birb_vision_nest::bindings;
use clap::Parser;

/// Hello
///
/// there
#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,

    /// The log level
    ///
    /// Valid values are: "off", "error", "warn", "info", "debug", "trace"
    #[clap(short, long, default_value = "warn")]
    log: clap_logger::LevelFilter,
}


#[derive(Debug, Parser)]
enum Command {
    Check(Check),
}

/// Hello
///
/// there
#[derive(Debug, Clone, Parser)]
#[clap(styles(cli_styles::CLAP_STYLES))]
struct Check {
    /// The path to the shared library to test
    shared_library: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.log)
        .init();

    let Command::Check(args) = args.command;


    let api = bindings::load(args.shared_library)?;
    println!("Loaded API: {api} with version: {:?}", api.get_version());

    let layers = api.supported_transport_layers().map_err(|e| anyhow::anyhow!("Failed to get supported transport layers: {}", e))?;
    println!("Supported transport layers: {:?}", layers);

    log::debug!("Dropping the API...");
    drop(api);
    log::debug!("API dropped");

    println!("Ok, LGTM.");
    Ok(())
}