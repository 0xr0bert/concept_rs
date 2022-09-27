mod json;

use std::{
    fs::File,
    io,
    ptr::{null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use anyhow::{Context, Result};
use belief_spread::{Agent, BasicBehaviour, BasicBelief, Behaviour, Belief, SimTime};
use clap::Parser;
use json::{BehaviourSpec, BeliefSpec};

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
        long = "behaviours",
        default_value = "behaviours.json"
    )]
    behaviours_file: std::path::PathBuf,

    /// The beliefs.json file
    #[clap(
        parse(from_os_str),
        short = 'c',
        long = "beliefs",
        default_value = "beliefs.json"
    )]
    beliefs_file: std::path::PathBuf,
}

/// The configuration of the model.
struct Configuration {
    /// The [Behaviour]s in the model.
    behaviours: *const [*const dyn Behaviour],

    /// The [Belief]s in the model.
    beliefs: *const [*const dyn Belief],

    /// The [Agent]s in the model.
    agents: *const [*const dyn Agent],

    /// The mutable [Behaviour]s in the model.
    behaviours_mut: *mut [*mut dyn Behaviour],

    /// The mutable [Belief]s in the model.
    beliefs_mut: *mut [*mut dyn Belief],

    /// The mutable [Agent]s in the model.
    agents_mut: *mut [*mut dyn Agent],
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut config: Box<Configuration> = Box::new(Configuration {
        behaviours: slice_from_raw_parts(null(), 0),
        beliefs: slice_from_raw_parts(null(), 0),
        agents: slice_from_raw_parts(null(), 0),
        behaviours_mut: slice_from_raw_parts_mut(null_mut(), 0),
        beliefs_mut: slice_from_raw_parts_mut(null_mut(), 0),
        agents_mut: slice_from_raw_parts_mut(null_mut(), 0),
    });

    // Process behaviours

    let mut behaviours = read_behaviours_json(&args.behaviours_file)?;

    let mut behaviours_ptrs_mut: Vec<*mut dyn Behaviour> = behaviours
        .iter_mut()
        .map(|b| b as *mut dyn Behaviour)
        .collect();

    let behaviours_ptrs_mut_slice: &mut [*mut dyn Behaviour] = &mut behaviours_ptrs_mut;

    config.behaviours_mut = behaviours_ptrs_mut_slice;

    let behaviours_ptrs: Vec<*const dyn Behaviour> = behaviours
        .iter()
        .map(|b| b as *const dyn Behaviour)
        .collect();

    let behaviours_ptrs_slice: &[*const dyn Behaviour] = &behaviours_ptrs;

    config.behaviours = behaviours_ptrs_slice;

    // Process beliefs

    let (belief_specs, mut beliefs) = read_belief_json(&args.beliefs_file, &config)?;

    let mut beliefs_ptrs_mut: Vec<*mut dyn Belief> =
        beliefs.iter_mut().map(|b| b as *mut dyn Belief).collect();

    let beliefs_ptr_mut_slice: &mut [*mut dyn Belief] = &mut beliefs_ptrs_mut;

    config.beliefs_mut = beliefs_ptr_mut_slice;

    let beliefs_ptrs: Vec<*const dyn Belief> =
        beliefs.iter().map(|b| b as *const dyn Belief).collect();

    let beliefs_ptrs_slice: &[*const dyn Belief] = &&beliefs_ptrs;

    config.beliefs = beliefs_ptrs_slice;

    belief_specs
        .iter()
        .for_each(|b| unsafe { b.link_belief_relationships(config.beliefs_mut) });

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

fn read_belief_json(
    path: &std::path::Path,
    config: &Configuration,
) -> Result<(Vec<BeliefSpec>, Vec<BasicBelief>)> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read beliefs from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let belief_specs: Vec<BeliefSpec> =
        serde_json::from_reader(reader).with_context(|| "beliefs.json invalid")?;
    let beliefs = belief_specs
        .iter()
        .map(|spec| unsafe { spec.to_basic_belief(config.behaviours) })
        .collect();
    Ok((belief_specs, beliefs))
}
