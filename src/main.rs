mod json;
mod performance_relationships;

use std::{
    collections::HashMap,
    fs::File,
    io,
    ptr::{null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use anyhow::{Context, Result};
use belief_spread::{
    Agent, BasicAgent, BasicBehaviour, BasicBelief, Behaviour, Belief, SimTime, UUIDd,
};
use clap::Parser;
use json::{AgentSpec, BehaviourSpec, BeliefSpec, PerformanceRelationshipSpec};
use performance_relationships::{vec_prs_to_performance_relationships, PerformanceRelationships};
use uuid::Uuid;

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

    /// The agents.json file
    #[clap(
        parse(from_os_str),
        short = 'a',
        long = "agents",
        default_value = "agents.json"
    )]
    agents_file: std::path::PathBuf,

    /// The prs.json file
    #[clap(
        parse(from_os_str),
        short = 'p',
        long = "performance-relationships",
        default_value = "prs.json"
    )]
    prs_file: std::path::PathBuf,
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

    /// The performance relationships in the model.
    prs: PerformanceRelationships,
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
        prs: HashMap::new(),
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

    // Process agents

    let (agent_specs, mut agents) =
        read_agent_json(&args.agents_file, config.beliefs, config.behaviours)?;

    let mut agents_ptrs_mut: Vec<*mut dyn Agent> =
        agents.iter_mut().map(|a| a as *mut dyn Agent).collect();

    let agents_ptr_mut_slice: &mut [*mut dyn Agent] = &mut agents_ptrs_mut;

    config.agents_mut = agents_ptr_mut_slice;

    let agents_ptrs: Vec<*const dyn Agent> = agents.iter().map(|a| a as *const dyn Agent).collect();

    let agent_ptrs_slice: &[*const dyn Agent] = &agents_ptrs;

    config.agents = agent_ptrs_slice;

    let uuid_agents: HashMap<Uuid, *mut dyn Agent> = agents
        .iter_mut()
        .map(|a| (a.uuid().clone(), a as *mut dyn Agent))
        .collect();

    agent_specs
        .iter()
        .for_each(|spec| unsafe { spec.link_friends(&uuid_agents) });

    // Process performance relationships

    unsafe {
        config.prs = read_prs_json(&args.prs_file, config.beliefs, config.behaviours)?;
    }

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

fn read_agent_json(
    path: &std::path::Path,
    beliefs: *const [*const dyn Belief],
    behaviours: *const [*const dyn Behaviour],
) -> Result<(Vec<AgentSpec>, Vec<BasicAgent>)> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read agents from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let agent_specs: Vec<AgentSpec> =
        serde_json::from_reader(reader).with_context(|| "agents.json invalid")?;
    let agents = agent_specs
        .iter()
        .map(|spec| unsafe { spec.to_basic_agent(behaviours, beliefs) })
        .collect();
    Ok((agent_specs, agents))
}

unsafe fn read_prs_json(
    path: &std::path::Path,
    beliefs: *const [*const dyn Belief],
    behaviours: *const [*const dyn Behaviour],
) -> Result<PerformanceRelationships> {
    let file = File::open(path).with_context(|| {
        format!(
            "Failed to read performance relationships from {}",
            path.display()
        )
    })?;
    let reader = io::BufReader::new(file);
    let prss: Vec<PerformanceRelationshipSpec> =
        serde_json::from_reader(reader).with_context(|| "prs.json invalid")?;
    let uuid_beliefs: HashMap<Uuid, *const dyn Belief> = beliefs
        .as_ref()
        .unwrap()
        .iter()
        .map(|b| (b.as_ref().unwrap().uuid().clone(), *b))
        .collect();

    let uuid_behaviours: HashMap<Uuid, *const dyn Behaviour> = behaviours
        .as_ref()
        .unwrap()
        .iter()
        .map(|b| (b.as_ref().unwrap().uuid().clone(), *b))
        .collect();
    Ok(vec_prs_to_performance_relationships(
        &prss,
        &uuid_beliefs,
        &uuid_behaviours,
    ))
}
