use std::collections::HashMap;

use belief_spread::{
    Agent, AgentPtr, BasicAgent, BasicBehaviour, BasicBelief, BehaviourPtr, Belief, BeliefPtr,
    SimTime,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The specification for a JSON file representing behaviours.
#[derive(Deserialize, Serialize, Debug)]
pub struct BehaviourSpec {
    /// The name of the behaviour.
    pub name: String,
    /// The UUID of the behaviour.
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
}

impl BehaviourSpec {
    /// Convert this [BehaviourSpec] into a [BasicBehaviour].
    pub fn to_basic_behaviour(&self) -> BasicBehaviour {
        BasicBehaviour::new_with_uuid(self.name.clone(), self.uuid)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BeliefSpec {
    pub name: String,
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    #[serde(default = "HashMap::new")]
    pub perceptions: HashMap<Uuid, f64>,
    #[serde(default = "HashMap::new")]
    pub relationships: HashMap<Uuid, f64>,
}

impl BeliefSpec {
    pub fn to_basic_belief(&self, behaviours: &[BehaviourPtr]) -> BeliefPtr {
        let mut b = BasicBelief::new_with_uuid(self.name.clone(), self.uuid);
        behaviours.iter().for_each(|beh| {
            if let Some(&v) = self.perceptions.get(beh.borrow().uuid()) {
                b.set_perception(beh.clone(), Some(v)).unwrap()
            }
        });
        b.into()
    }

    pub fn link_belief_relationships(&self, beliefs: &[BeliefPtr]) {
        let uuid_beliefs: HashMap<Uuid, &BeliefPtr> =
            beliefs.iter().map(|b| (*b.borrow().uuid(), b)).collect();
        self.relationships.iter().for_each(|(r, &v)| {
            if let Some(b) = uuid_beliefs.get(r) {
                uuid_beliefs
                    .get(&self.uuid)
                    .unwrap()
                    .borrow_mut()
                    .set_relationship((*b).clone(), Some(v))
                    .unwrap()
            }
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AgentSpec {
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    #[serde(default = "HashMap::new")]
    pub actions: HashMap<SimTime, Uuid>,
    #[serde(default = "HashMap::new")]
    pub activations: HashMap<SimTime, HashMap<Uuid, f64>>,
    #[serde(default = "HashMap::new")]
    pub deltas: HashMap<Uuid, f64>,
    #[serde(default = "HashMap::new")]
    pub friends: HashMap<Uuid, f64>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceRelationshipSpec {
    pub behaviour_uuid: Uuid,
    pub belief_uuid: Uuid,
    pub value: f64,
}

impl AgentSpec {
    pub fn to_basic_agent(&self, behaviours: &[BehaviourPtr], beliefs: &[BeliefPtr]) -> AgentPtr {
        let mut a = BasicAgent::new_with_uuid(self.uuid);
        let uuid_behaviours: HashMap<Uuid, &BehaviourPtr> =
            behaviours.iter().map(|b| (*b.borrow().uuid(), b)).collect();

        self.actions.iter().for_each(|(&time, b)| {
            a.set_action(time, Some((*uuid_behaviours.get(b).unwrap()).clone()))
        });

        let uuid_beliefs: HashMap<Uuid, &BeliefPtr> =
            beliefs.iter().map(|b| (*b.borrow().uuid(), b)).collect();

        self.activations.iter().for_each(|(&time, acts)| {
            acts.iter().for_each(|(b, &v)| {
                a.set_activation(time, (*uuid_beliefs.get(b).unwrap()).clone(), Some(v))
                    .unwrap()
            })
        });

        self.deltas.iter().for_each(|(b, &v)| {
            a.set_delta((*uuid_beliefs.get(b).unwrap()).clone(), Some(v))
                .unwrap()
        });

        a.into()
    }

    pub fn link_friends(&self, agents: &HashMap<Uuid, AgentPtr>) {
        let mut this_agent = agents.get(&self.uuid).unwrap().borrow_mut();

        self.friends.iter().for_each(|(a, &v)| {
            this_agent
                .set_friend_weight((*agents.get(a).unwrap()).clone(), Some(v))
                .unwrap()
        });
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSpec {
    pub mean_activation: HashMap<Uuid, f64>,
    pub sd_activation: HashMap<Uuid, f64>,
    pub median_activation: HashMap<Uuid, f64>,
    pub nonzero_activation_count: HashMap<Uuid, usize>,
    pub n_performers: HashMap<Uuid, usize>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSpecs {
    pub data: HashMap<SimTime, OutputSpec>,
}

impl OutputSpecs {
    pub fn from_agents(
        agents: &[AgentPtr],
        beliefs: &[BeliefPtr],
        start_time: SimTime,
        end_time: SimTime,
    ) -> Self {
        let data: HashMap<SimTime, OutputSpec> = (start_time..=end_time)
            .map(|t| {
                // Calculate avg_activation
                let mut mean_activation: HashMap<Uuid, f64> = HashMap::new();

                for agent in agents {
                    if let Some(m) = agent.borrow().get_activations().get(&t) {
                        for (belief, activation) in m {
                            let entry = mean_activation
                                .entry(*belief.borrow().uuid())
                                .or_insert(0.0);
                            *entry += activation;
                        }
                    }
                }

                let n_agents = agents.len();
                for (_, activation) in mean_activation.iter_mut() {
                    *activation /= n_agents as f64;
                }

                // Calculate sd_activation
                let mut sd_activation: HashMap<Uuid, f64> = HashMap::new();

                for agent in agents {
                    if let Some(m) = agent.borrow().get_activations().get(&t) {
                        for (belief, activation) in m {
                            let entry = sd_activation.entry(*belief.borrow().uuid()).or_insert(0.0);

                            *entry += f64::powf(
                                activation - mean_activation.get(belief.borrow().uuid()).unwrap(),
                                2.0,
                            )
                        }
                    }
                }

                for (_, sd) in sd_activation.iter_mut() {
                    *sd = f64::sqrt(*sd / ((n_agents - 1) as f64));
                }

                // Calculate median activation
                let mut activations_by_uuid: HashMap<Uuid, Vec<f64>> = HashMap::new();

                for agent in agents {
                    let agent_ptr = agent.borrow();
                    for belief in beliefs {
                        let entry = activations_by_uuid
                            .entry(*belief.borrow().uuid())
                            .or_insert_with(Vec::new);
                        entry.push(agent_ptr.get_activation(t, belief).unwrap_or(0.0));
                    }
                }

                let middle_index = n_agents / 2;

                let mut median_activation: HashMap<Uuid, f64> = HashMap::new();

                for (uuid, mut acts) in activations_by_uuid {
                    acts.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                    median_activation.insert(uuid, *acts.get(middle_index).unwrap());
                }

                // Calculate non_zero activation count
                let mut nonzero_activation_count: HashMap<Uuid, usize> = HashMap::new();

                for agent in agents {
                    if let Some(m) = agent.borrow().get_activations().get(&t) {
                        for (belief, activation) in m {
                            if *activation != 0.0 {
                                let entry = nonzero_activation_count
                                    .entry(*belief.borrow().uuid())
                                    .or_insert(0);
                                *entry += 1;
                            }
                        }
                    }
                }

                // Calculate n_performers
                let n_performers: HashMap<Uuid, usize> = agents
                    .iter()
                    .flat_map(|a| {
                        a.borrow()
                            .get_action(t)
                            .map(|action| *action.borrow().uuid())
                    })
                    .fold(HashMap::new(), |mut counts, elem| {
                        let count = counts.entry(elem).or_insert(0);
                        *count += 1;
                        counts
                    });
                (
                    t,
                    OutputSpec {
                        mean_activation,
                        sd_activation,
                        median_activation,
                        nonzero_activation_count,
                        n_performers,
                    },
                )
            })
            .collect();

        Self { data }
    }
}

mod test {
    #[cfg(test)]
    mod behaviour_spec {
        use super::super::*;

        use belief_spread::{Named, UUIDd};

        #[test]
        fn valid_name_and_uuid() {
            let json_str = r#"
            {
                "name": "Behaviour 1",
                "uuid": "98f4a478-7deb-40ef-9cb5-0f893c7a7f45"
            }
            "#;

            let uuid = uuid::uuid!("98f4a478-7deb-40ef-9cb5-0f893c7a7f45");

            let b: BehaviourSpec = serde_json::from_str(json_str).unwrap();
            assert_eq!(b.name, "Behaviour 1");
            assert_eq!(b.uuid, uuid);
        }

        #[test]
        fn valid_name_and_unspecified_uuid() {
            let json_str = r#"
            {
                "name": "Behaviour 1"
            }
            "#;

            let uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");

            let b: BehaviourSpec = serde_json::from_str(json_str).unwrap();
            assert_eq!(b.name, "Behaviour 1");
            assert_ne!(b.uuid, uuid);
        }

        #[test]
        fn invalid_name_and_valid_uuid() {
            let json_str = r#"
            {
                "name": 2,
                "uuid": "98f4a478-7deb-40ef-9cb5-0f893c7a7f45"
            }
            "#;

            assert!(serde_json::from_str::<BehaviourSpec>(json_str).is_err());
        }

        #[test]
        fn invalid_name_and_unspecified_uuid() {
            let json_str = r#"
            {
                "name": 2
            }
            "#;

            assert!(serde_json::from_str::<BehaviourSpec>(json_str).is_err());
        }

        #[test]
        fn valid_name_and_invalid_uuid() {
            let json_str = r#"
            {
                "name": "Behaviour 1",
                "uuid": "aaa"
            }
            "#;

            assert!(serde_json::from_str::<BehaviourSpec>(json_str).is_err());
        }

        #[test]
        fn array_of_valid_behaviour_specs() {
            let json_str = r#"
            [
                {
                    "name": "Behaviour 1",
                    "uuid": "98f4a478-7deb-40ef-9cb5-0f893c7a7f45"
                },
                {
                    "name": "Behaviour 2"
                }
            ]
            "#;

            let uuid = uuid::uuid!("98f4a478-7deb-40ef-9cb5-0f893c7a7f45");

            let b: Vec<BehaviourSpec> = serde_json::from_str(json_str).unwrap();
            assert_eq!(b.len(), 2);
            assert_eq!(b.get(0).unwrap().name, "Behaviour 1");
            assert_eq!(b.get(0).unwrap().uuid, uuid);
            assert_eq!(b.get(1).unwrap().name, "Behaviour 2");
            assert_ne!(b.get(0).unwrap().uuid, b.get(1).unwrap().uuid);
            let zero_uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");
            assert_ne!(b.get(1).unwrap().uuid, zero_uuid)
        }

        #[test]
        fn to_basic_behaviour_works() {
            let u = Uuid::new_v4();
            let bi = BehaviourSpec {
                name: "b1".to_string(),
                uuid: u,
            };
            let bo = bi.to_basic_behaviour();
            assert_eq!(bo.name(), "b1");
            assert_eq!(bo.uuid(), &u);
        }
    }
}
