mod json;

use std::{fs::File, io};

use anyhow::{Context, Result};
use belief_spread::{BasicBehaviour, SimTime};
use clap::Parser;
use json::BehaviourSpec;

/// The arguments of the command-line interface
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The start time of the simulation
    #[clap(short = 's', long = "start", value_parser, default_value_t = 1)]
    start_time: SimTime,

    /// The end time of the simulation
    #[clap(short = 'e', long = "end", value_parser, default_value_t = 1)]
    end_time: SimTime,

    /// The output file
    #[clap(
        parse(from_os_str),
        short = 'o',
        long = "output",
        default_value = "output.json"
    )]
    output_file: std::path::PathBuf,

    /// The behaviours.json file
    #[clap(
        parse(from_os_str),
        short = 'b',
        long = "behaviours.json",
        default_value = "behaviours.json"
    )]
    behaviours_file: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let _behaviours = read_behaviours_json(&args.behaviours_file)?;

    Ok(())
}

fn read_behaviours_json(path: &std::path::Path) -> Result<Vec<BasicBehaviour>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read behaviours from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let behaviours: Vec<BehaviourSpec> =
        serde_json::from_reader(reader).with_context(|| "behaviours.json invalid")?;
    Ok(behaviours
        .into_iter()
        .map(|spec| spec.to_basic_behaviour())
        .collect())
}
