use belief_spread::{Behaviour, Belief};
use std::collections::HashMap;
use uuid::Uuid;

use crate::json::PerformanceRelationshipSpec;

/// The value is how much someone holding the [Belief] would like to perform
/// the [Behaviour].
pub type PerformanceRelationships = HashMap<(*const dyn Belief, *const dyn Behaviour), f64>;

/// Convert [PerformanceRelationshipSpec]s to [PerformanceRelationships].
///
/// # Arguments
/// - `prss`: The [PerformanceRelationshipSpec].
/// - `belief`: The [Belief]s mapped from their [Uuid]s.
/// - `behaviour`: The [Behaviour]s mapped from their [Uuid]s.
///
/// # Returns
/// The [PerformanceRelationships].
pub fn vec_prs_to_performance_relationships(
    prss: &[PerformanceRelationshipSpec],
    beliefs: &HashMap<Uuid, *const dyn Belief>,
    behaviours: &HashMap<Uuid, *const dyn Behaviour>,
) -> PerformanceRelationships {
    prss.iter()
        .map(|prs| {
            (
                (
                    *beliefs.get(&prs.belief_uuid).unwrap(),
                    *behaviours.get(&prs.behaviour_uuid).unwrap(),
                ),
                prs.value,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use belief_spread::{BasicBehaviour, BasicBelief, UUIDd};
    use uuid::Uuid;

    use crate::json::PerformanceRelationshipSpec;

    use super::*;

    #[test]
    fn test_vec_prs_to_performance_relationships_works() {
        let mut prss: Vec<PerformanceRelationshipSpec> = Vec::new();
        let belief = BasicBelief::new("b1".to_string());
        let behaviour = BasicBehaviour::new("b1".to_string());
        prss.push(PerformanceRelationshipSpec {
            behaviour_uuid: *behaviour.uuid(),
            belief_uuid: *belief.uuid(),
            value: 0.2,
        });
        let mut beliefs: HashMap<Uuid, *const dyn Belief> = HashMap::new();
        let belief_ptr: *const dyn Belief = &belief;
        unsafe {
            beliefs.insert(*(*belief_ptr).uuid(), belief_ptr);
        }

        let mut behaviours: HashMap<Uuid, *const dyn Behaviour> = HashMap::new();
        let behaviour_ptr: *const dyn Behaviour = &behaviour;
        unsafe {
            behaviours.insert(*(*behaviour_ptr).uuid(), behaviour_ptr);
        }

        let result = vec_prs_to_performance_relationships(&prss, &beliefs, &behaviours);
        assert_eq!(result.len(), 1);
        assert_eq!(*result.get(&(belief_ptr, behaviour_ptr)).unwrap(), 0.2)
    }
}
