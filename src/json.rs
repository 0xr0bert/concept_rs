use std::collections::HashMap;

use belief_spread::{Agent, BasicAgent, BasicBehaviour, BasicBelief, Behaviour, Belief, SimTime};
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
    pub fn to_basic_behaviour(self) -> BasicBehaviour {
        BasicBehaviour::new_with_uuid(self.name, self.uuid)
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
    pub unsafe fn to_basic_belief(&self, behaviours: *const [*const dyn Behaviour]) -> BasicBelief {
        let mut b = BasicBelief::new_with_uuid(self.name.clone(), self.uuid);
        behaviours.as_ref().unwrap().iter().for_each(|beh| {
            match self.perceptions.get(beh.as_ref().unwrap().uuid()) {
                Some(&v) => b.set_perception(*beh, Some(v)).unwrap(),
                None => (),
            }
        });
        b
    }

    pub unsafe fn link_belief_relationships(&self, beliefs: *mut [*mut dyn Belief]) {
        let uuid_beliefs: HashMap<&Uuid, *mut dyn Belief> = beliefs
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|b| (b.as_ref().unwrap().uuid(), *b))
            .collect();
        self.relationships
            .iter()
            .for_each(|(r, &v)| match uuid_beliefs.get(&r) {
                Some(b) => uuid_beliefs
                    .get(&self.uuid)
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .set_relationship(*b, Some(v))
                    .unwrap(),
                None => (),
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
    pub unsafe fn to_basic_agent(
        &self,
        behaviours: *const [*const dyn Behaviour],
        beliefs: *const [*const dyn Belief],
    ) -> BasicAgent {
        let mut a = BasicAgent::new_with_uuid(self.uuid);
        let uuid_behaviours: HashMap<&Uuid, *const dyn Behaviour> = behaviours
            .as_ref()
            .unwrap()
            .iter()
            .map(|b| (b.as_ref().unwrap().uuid(), *b))
            .collect();

        self.actions
            .iter()
            .for_each(|(&time, b)| a.set_action(time, Some(*uuid_behaviours.get(&b).unwrap())));

        let uuid_beliefs: HashMap<&Uuid, *const dyn Belief> = beliefs
            .as_ref()
            .unwrap()
            .iter()
            .map(|b| (b.as_ref().unwrap().uuid(), *b))
            .collect();

        self.activations.iter().for_each(|(&time, acts)| {
            acts.iter().for_each(|(b, &v)| {
                a.set_activation(time, *uuid_beliefs.get(&b).unwrap(), Some(v))
                    .unwrap()
            })
        });

        self.deltas.iter().for_each(|(b, &v)| {
            a.set_delta(*uuid_beliefs.get(&b).unwrap(), Some(v))
                .unwrap()
        });

        a
    }

    pub unsafe fn link_friends(&self, agents: &HashMap<Uuid, *mut dyn Agent>) {
        let this_agent = agents.get(&self.uuid).unwrap().as_mut().unwrap();

        self.friends.iter().for_each(|(a, &v)| {
            this_agent
                .set_friend_weight(*agents.get(a).unwrap(), Some(v))
                .unwrap()
        });
    }

    pub unsafe fn fromAgent(agent: &dyn Agent) -> Self {
        AgentSpec {
            uuid: agent.uuid().clone(),
            actions: agent
                .get_actions()
                .iter()
                .map(|(&k, v)| (k, v.as_ref().unwrap().uuid().clone()))
                .collect(),
            activations: agent
                .get_activations()
                .iter()
                .map(|(&k1, v1)| {
                    (
                        k1,
                        v1.iter()
                            .map(|(k2, &v2)| (k2.as_ref().unwrap().uuid().clone(), v2))
                            .collect(),
                    )
                })
                .collect(),
            deltas: agent
                .get_deltas()
                .iter()
                .map(|(k, &v)| (k.as_ref().unwrap().uuid().clone(), v))
                .collect(),
            friends: agent
                .get_friends()
                .iter()
                .map(|(k, &v)| (k.as_ref().unwrap().uuid().clone(), v))
                .collect(),
        }
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
                uuid: u.clone(),
            };
            let bo = bi.to_basic_behaviour();
            assert_eq!(bo.name(), "b1");
            assert_eq!(bo.uuid(), &u);
        }
    }
}
