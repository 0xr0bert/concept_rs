use belief_spread::{Behaviour, Belief};
use std::collections::HashMap;
use uuid::Uuid;

use crate::json::PerformanceRelationshipSpec;

pub type PerformanceRelationships = HashMap<(*const dyn Belief, *const dyn Behaviour), f64>;

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
