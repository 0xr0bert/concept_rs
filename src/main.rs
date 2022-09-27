use belief_spread::SimTime;
use clap::Parser;

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
}

fn main() {
    let _args = Cli::parse();
}
