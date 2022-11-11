use belief_spread::{BehaviourPtr, BeliefPtr};
use std::collections::HashMap;
use uuid::Uuid;

use crate::json::PerformanceRelationshipSpec;

/// The value is how much someone holding the [Belief] would like to perform
/// the [Behaviour].
pub type PerformanceRelationships = HashMap<(BeliefPtr, BehaviourPtr), f64>;

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
    beliefs: &HashMap<Uuid, BeliefPtr>,
    behaviours: &HashMap<Uuid, BehaviourPtr>,
) -> PerformanceRelationships {
    prss.iter()
        .map(|prs| {
            (
                (
                    beliefs.get(&prs.belief_uuid).unwrap().clone(),
                    behaviours.get(&prs.behaviour_uuid).unwrap().clone(),
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
        let mut beliefs: HashMap<Uuid, BeliefPtr> = HashMap::new();
        let belief_ptr = BeliefPtr::from(belief);
        beliefs.insert(*belief_ptr.borrow().uuid(), belief_ptr.clone());

        let mut behaviours: HashMap<Uuid, BehaviourPtr> = HashMap::new();
        let behaviour_ptr = BehaviourPtr::from(behaviour);
        behaviours.insert(*behaviour_ptr.borrow().uuid(), behaviour_ptr.clone());

        let result = vec_prs_to_performance_relationships(&prss, &beliefs, &behaviours);
        assert_eq!(result.len(), 1);
        assert_eq!(*result.get(&(belief_ptr, behaviour_ptr)).unwrap(), 0.2)
    }
}
