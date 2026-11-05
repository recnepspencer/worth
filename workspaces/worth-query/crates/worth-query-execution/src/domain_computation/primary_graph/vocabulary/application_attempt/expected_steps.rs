//! Moved whole from `domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::expected_steps`: the items below and every impl they own.

/// Preparation cardinality: one map lookup per lowered step and one exact
/// equality check per repeated identity. These are not map-comparison counts
/// or, by themselves, proof of the lookup algorithm's complexity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryExpectedEffectStepPreparationWork
{
    key_lookups: usize,
    duplicate_equalities: usize,
}

impl WorthQueryExpectedEffectStepPreparationWork {
    pub(in crate::domain_computation::primary_graph) const fn key_lookups(self) -> usize {
        self.key_lookups
    }

    pub(in crate::domain_computation::primary_graph) const fn duplicate_equalities(self) -> usize {
        self.duplicate_equalities
    }
}
