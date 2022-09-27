use belief_spread::BasicBehaviour;
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
