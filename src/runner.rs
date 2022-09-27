use std::collections::HashMap;

use anyhow::Result;
use belief_spread::{Agent, Behaviour, SimTime};
use log::info;
use rand::Rng;

use crate::{json::AgentSpec, Configuration};

pub struct Runner {
    pub config: Box<Configuration>,
}

impl Runner {
    pub unsafe fn run(&mut self) -> Result<()> {
        simple_logger::init_with_env().unwrap();
        info!("Starting concept");
        info!("n beliefs: {}", self.config.beliefs.as_ref().unwrap().len());
        info!(
            "n behaviours: {}",
            self.config.behaviours.as_ref().unwrap().len()
        );
        info!("n agents: {}", self.config.agents.as_ref().unwrap().len());
        info!("Start time: {}", self.config.start_time);
        info!("End time: {}", self.config.end_time);
        self.tick_between(self.config.start_time, self.config.end_time);
        info!("Ending concept");
        self.serialize_agents()?;
        Ok(())
    }

    pub unsafe fn serialize_agents(&mut self) -> Result<()> {
        info!("Converting agents to AgentSpecs");
        let specs: Vec<AgentSpec> = self
            .config
            .agents
            .as_ref()
            .unwrap()
            .iter()
            .map(|a| AgentSpec::from_agent(a.as_ref().unwrap()))
            .collect();

        info!("Writing agents to file");
        let writer = std::io::BufWriter::new(&mut self.config.output_file);
        serde_json::to_writer(writer, &specs)?;
        Ok(())
    }

    unsafe fn tick_between(&mut self, start: SimTime, end: SimTime) {
        for t in start..=end {
            self.tick(t);
        }
    }

    unsafe fn tick(&mut self, time: SimTime) {
        info!("Day {time} - perceiving beliefs");
        self.perceive_beliefs(time);
        info!("Day {time} - performing actions");
        self.perform_actions(time);
    }

    unsafe fn perceive_beliefs(&mut self, time: SimTime) {
        self.config
            .agents_mut
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|a| {
                self.config.beliefs.as_ref().unwrap().iter().for_each(|b| {
                    a.as_mut()
                        .unwrap()
                        .update_activation(time, *b, self.config.beliefs)
                        .unwrap();
                })
            });
    }

    unsafe fn agent_perform_action(&mut self, agent: &mut dyn Agent, time: SimTime) {
        let mut unnormalized_probs: Vec<(*const dyn Behaviour, f64)> = self
            .config
            .behaviours
            .as_ref()
            .unwrap()
            .iter()
            .map(|&behaviour| {
                (
                    behaviour,
                    self.config
                        .beliefs
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|&belief| {
                            self.config.prs.get(&(belief, behaviour)).unwrap_or(&0.0)
                                * agent
                                    .get_activation(time, belief.as_ref().unwrap())
                                    .unwrap_or(0.0)
                        })
                        .sum::<f64>(),
                )
            })
            .collect();
        unnormalized_probs.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

        match unnormalized_probs.last().unwrap() {
            (k, v) if *v <= 0.0 => agent.set_action(time, Some(*k)),
            _ => {
                let filtered_probs: Vec<(*const dyn Behaviour, f64)> = unnormalized_probs
                    .into_iter()
                    .filter(|(_, x)| *x > 0.0)
                    .collect();
                match filtered_probs.len() {
                    1 => agent.set_action(time, Some(filtered_probs.get(0).unwrap().0)),
                    _ => {
                        let map_probs: HashMap<*const dyn Behaviour, f64> =
                            filtered_probs.into_iter().collect();
                        let normalizing_factor: f64 = map_probs.values().sum();
                        let normalized_probs: Vec<(*const dyn Behaviour, f64)> = map_probs
                            .into_iter()
                            .map(|(k, v)| (k, v / normalizing_factor))
                            .collect();

                        let mut rng = rand::thread_rng();
                        let mut rv: f64 = rng.gen();
                        let mut chosen_behaviour = normalized_probs.last().unwrap().0;

                        for (behaviour, v) in normalized_probs.into_iter() {
                            rv -= v;
                            if rv <= 0.0 {
                                chosen_behaviour = behaviour;
                                break;
                            }
                        }

                        agent.set_action(time, Some(chosen_behaviour))
                    }
                }
            }
        }
    }

    unsafe fn perform_actions(&mut self, time: SimTime) {
        self.config
            .agents_mut
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|agent| self.agent_perform_action(agent.as_mut().unwrap(), time));
    }
}
