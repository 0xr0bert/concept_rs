use std::collections::HashMap;

use anyhow::Result;
use belief_spread::{update_activation_for_all_beliefs_for_agent, AgentPtr, BehaviourPtr, SimTime};
use log::info;
use rand::Rng;

use crate::{json::AgentSpec, Configuration};

pub struct Runner {
    pub config: Box<Configuration>,
}

impl Runner {
    pub fn run(&mut self) -> Result<()> {
        simple_logger::init_with_env().unwrap();
        info!("Starting concept");
        info!("n beliefs: {}", self.config.beliefs.len());
        info!("n behaviours: {}", self.config.behaviours.len());
        info!("n agents: {}", self.config.agents.len());
        info!("Start time: {}", self.config.start_time);
        info!("End time: {}", self.config.end_time);
        self.tick_between(self.config.start_time, self.config.end_time);
        info!("Ending concept");
        self.serialize_agents()?;
        Ok(())
    }

    pub fn serialize_agents(&mut self) -> Result<()> {
        info!("Converting agents to AgentSpecs");
        let specs: Vec<AgentSpec> = self
            .config
            .agents
            .iter()
            .map(AgentSpec::from_agent)
            .collect();

        info!("Writing agents to file");
        let writer = std::io::BufWriter::new(&mut self.config.output_file);
        let writer_zstd = zstd::stream::write::Encoder::new(writer, 3)?;
        serde_json::to_writer(writer_zstd, &specs)?;
        Ok(())
    }

    fn tick_between(&mut self, start: SimTime, end: SimTime) {
        for t in start..=end {
            self.tick(t);
        }
    }

    fn tick(&mut self, time: SimTime) {
        info!("Day {time} - perceiving beliefs");
        self.perceive_beliefs(time);
        info!("Day {time} - performing actions");
        self.perform_actions(time);
    }

    fn perceive_beliefs(&mut self, time: SimTime) {
        for a in self.config.agents.iter() {
            update_activation_for_all_beliefs_for_agent(a, time, &self.config.beliefs).unwrap();
        }
    }

    fn agent_perform_action(&self, agent: &AgentPtr, time: SimTime) {
        let mut unnormalized_probs: Vec<(BehaviourPtr, f64)> = self
            .config
            .behaviours
            .iter()
            .map(|behaviour| {
                (
                    behaviour.clone(),
                    self.config
                        .beliefs
                        .iter()
                        .map(|belief| {
                            self.config
                                .prs
                                .get(&(belief.clone(), behaviour.clone()))
                                .unwrap_or(&0.0)
                                * agent.borrow().get_activation(time, belief).unwrap_or(0.0)
                        })
                        .sum::<f64>(),
                )
            })
            .collect();
        unnormalized_probs.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

        match unnormalized_probs.last().unwrap() {
            (k, v) if *v <= 0.0 => agent.borrow_mut().set_action(time, Some(k.clone())),
            _ => {
                let filtered_probs: Vec<(BehaviourPtr, f64)> = unnormalized_probs
                    .into_iter()
                    .filter(|(_, x)| *x > 0.0)
                    .collect();
                match filtered_probs.len() {
                    1 => agent
                        .borrow_mut()
                        .set_action(time, Some(filtered_probs.get(0).unwrap().0.clone())),
                    _ => {
                        let map_probs: HashMap<BehaviourPtr, f64> =
                            filtered_probs.into_iter().collect();
                        let normalizing_factor: f64 = map_probs.values().sum();
                        let normalized_probs: Vec<(BehaviourPtr, f64)> = map_probs
                            .into_iter()
                            .map(|(k, v)| (k, v / normalizing_factor))
                            .collect();

                        let mut rng = rand::thread_rng();
                        let mut rv: f64 = rng.gen();
                        let mut chosen_behaviour = normalized_probs.last().unwrap().0.clone();

                        for (behaviour, v) in normalized_probs.into_iter() {
                            rv -= v;
                            if rv <= 0.0 {
                                chosen_behaviour = behaviour;
                                break;
                            }
                        }

                        agent.borrow_mut().set_action(time, Some(chosen_behaviour))
                    }
                }
            }
        }
    }

    fn perform_actions(&mut self, time: SimTime) {
        self.config
            .agents
            .iter()
            .for_each(|agent| self.agent_perform_action(agent, time));
    }
}
