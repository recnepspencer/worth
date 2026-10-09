//! Computation input membership survives loss of reusable partition attribution.
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, WorthQueryApplicationFactKey as Key,
    WorthQueryApplicationObservedFact as Fact,
};
use std::collections::BTreeMap;

pub(in crate::domain_computation::primary_graph) enum ObservedComputationInputs {
    None,
    One(ComputationFactAttribution),
    Several(ComputationFactAttribution),
}
impl ObservedComputationInputs {
    pub(in crate::domain_computation::primary_graph) fn ordinals(
        &self,
        facts: &BTreeMap<Key, Fact>,
    ) -> Box<[usize]> {
        facts
            .keys()
            .enumerate()
            .filter_map(|(ordinal, key)| {
                let read = match self {
                    Self::None => false,
                    Self::One(attribution) | Self::Several(attribution) => {
                        attribution.contains_key(key)
                    }
                };
                read.then_some(ordinal)
            })
            .collect()
    }
    pub(in crate::domain_computation::primary_graph) fn into_attribution(
        self,
    ) -> Option<ComputationFactAttribution> {
        match self {
            Self::One(attribution) => Some(attribution),
            Self::None | Self::Several(_) => None,
        }
    }
}
