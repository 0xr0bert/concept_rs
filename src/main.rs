mod json;
mod performance_relationships;
mod runner;

use std::{collections::HashMap, fs::File, io};

use anyhow::{Context, Result};
use belief_spread::{Agent, Behaviour, Belief, SimTime};
use clap::Parser;
use json::{AgentSpec, BehaviourSpec, BeliefSpec, PerformanceRelationshipSpec};
use performance_relationships::{vec_prs_to_performance_relationships, PerformanceRelationships};
use runner::Runner;
use uuid::Uuid;

/// The arguments of the command-line interface
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The start time of the simulation
    #[clap(short = 's', long = "start", value_parser, default_value_t = 1)]
    start_time: SimTime,

    /// The end time of the simulation
    #[clap(short = 'e', long = "end", value_parser, default_value_t = 1)]
    end_time: SimTime,

    /// The output file
    #[arg(short = 'o', long = "output", default_value = "output.json.zst")]
    output_file: std::path::PathBuf,

    /// The behaviours.json file
    #[arg(short = 'b', long = "behaviours", default_value = "behaviours.json")]
    behaviours_file: std::path::PathBuf,

    /// The beliefs.json file
    #[arg(short = 'c', long = "beliefs", default_value = "beliefs.json")]
    beliefs_file: std::path::PathBuf,

    /// The agents.json file
    #[arg(short = 'a', long = "agents", default_value = "agents.json.zst")]
    agents_file: std::path::PathBuf,

    /// The prs.json file
    #[arg(
        short = 'p',
        long = "performance-relationships",
        default_value = "prs.json"
    )]
    prs_file: std::path::PathBuf,
}

/// The configuration of the model.
pub struct Configuration {
    /// The [Behaviour]s in the model.
    behaviours: Vec<*const dyn Behaviour>,

    /// The [Belief]s in the model.
    beliefs: Vec<*const dyn Belief>,

    /// The [Agent]s in the model.
    agents: Vec<*const dyn Agent>,

    /// The performance relationships in the model.
    prs: PerformanceRelationships,

    /// Start time.
    start_time: SimTime,

    /// End time.
    end_time: SimTime,

    /// Output file
    output_file: File,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut config: Box<Configuration> = Box::new(Configuration {
        behaviours: Vec::new(),
        beliefs: Vec::new(),
        agents: Vec::new(),
        prs: HashMap::new(),
        start_time: args.start_time,
        end_time: args.end_time,
        output_file: File::create(&args.output_file)
            .with_context(|| format!("File {} doesn't exist!", &args.output_file.display()))?,
    });

    unsafe {
        // Process behaviours

        config.behaviours = read_behaviours_json(&args.behaviours_file)?;

        // Process beliefs

        config.beliefs = read_belief_json(
            &args.beliefs_file,
            &config.behaviours as &[*const dyn Behaviour] as *const [*const dyn Behaviour],
        )?;

        // Process agents

        config.agents = read_agent_json(
            &args.agents_file,
            &config.beliefs as &[*const dyn Belief] as *const [*const dyn Belief],
            &config.behaviours as &[*const dyn Behaviour] as *const [*const dyn Behaviour],
        )?;

        // Process performance relationships

        config.prs = read_prs_json(
            &args.prs_file,
            &config.beliefs as &[*const dyn Belief] as *const [*const dyn Belief],
            &config.behaviours as &[*const dyn Behaviour] as *const [*const dyn Behaviour],
        )?;

        let mut run = Runner { config };

        run.run()?;

        // Free everything
        for behaviour in run.config.behaviours {
            drop(Box::from_raw(behaviour as *mut dyn Behaviour))
        }
        for belief in run.config.beliefs {
            drop(Box::from_raw(belief as *mut dyn Belief))
        }

        for agent in run.config.agents {
            drop(Box::from_raw(agent as *mut dyn Agent))
        }
    }

    Ok(())
}

fn read_behaviours_json(path: &std::path::Path) -> Result<Vec<*const dyn Behaviour>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read behaviours from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let behaviours: Vec<BehaviourSpec> =
        serde_json::from_reader(reader).with_context(|| "behaviours.json invalid")?;
    Ok(behaviours
        .into_iter()
        .map(|spec| spec.to_basic_behaviour())
        .map(Box::new)
        .map(Box::into_raw)
        .map(|a| a as *const dyn Behaviour)
        .collect())
}

unsafe fn read_belief_json(
    path: &std::path::Path,
    behaviours: *const [*const dyn Behaviour],
) -> Result<Vec<*const dyn Belief>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read beliefs from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let belief_specs: Vec<BeliefSpec> =
        serde_json::from_reader(reader).with_context(|| "beliefs.json invalid")?;
    let beliefs: Vec<*const dyn Belief> = belief_specs
        .iter()
        .map(|spec| spec.to_basic_belief(behaviours))
        .collect();

    belief_specs.iter().for_each(|spec| {
        spec.link_belief_relationships(
            &beliefs as &[*const dyn Belief] as *const [*const dyn Belief],
        )
    });
    Ok(beliefs)
}

unsafe fn read_agent_json(
    path: &std::path::Path,
    beliefs: *const [*const dyn Belief],
    behaviours: *const [*const dyn Behaviour],
) -> Result<Vec<*const dyn Agent>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to read agents from {}", path.display()))?;
    let reader = io::BufReader::new(file);
    let reader_zstd = zstd::stream::read::Decoder::new(reader)?;
    let agent_specs: Vec<AgentSpec> =
        serde_json::from_reader(reader_zstd).with_context(|| "agents.json invalid")?;
    let agents: Vec<*const dyn Agent> = agent_specs
        .iter()
        .map(|spec| spec.to_basic_agent(behaviours, beliefs))
        .collect();
    let uuid_agents: HashMap<Uuid, *const dyn Agent> =
        agents.iter().map(|a| (*(**a).uuid(), *a)).collect();

    agent_specs
        .iter()
        .for_each(|spec| spec.link_friends(&uuid_agents));

    Ok(agents)
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
    let uuid_beliefs: HashMap<Uuid, *const dyn Belief> =
        (*beliefs).iter().map(|b| (*(**b).uuid(), *b)).collect();

    let uuid_behaviours: HashMap<Uuid, *const dyn Behaviour> =
        (*behaviours).iter().map(|b| (*(**b).uuid(), *b)).collect();
    Ok(vec_prs_to_performance_relationships(
        &prss,
        &uuid_beliefs,
        &uuid_behaviours,
    ))
}
