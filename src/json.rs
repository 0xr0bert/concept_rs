use std::collections::HashMap;

use belief_spread::{BasicBehaviour, BasicBelief, Behaviour, Belief};
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