//! Exact producer rows and installed-family lifecycle succession.

use super::SourceEpoch;

use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability;
use std::{any::TypeId, cmp::Ordering};

// Family and source precede the binding: one output occurrence remains a
// contiguous range even when Initial and Preserve have separate executors.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandKey {
    pub(super) family: TypeId,
    pub(super) source: SourceEpoch,
    pub(super) applicability: WorthQueryProducerApplicability,
    pub(super) producer: String,
}

impl WorthQueryOutputDemandKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        family: TypeId,
        producer: String,
        applicability: WorthQueryProducerApplicability,
        source: SourceEpoch,
    ) -> Self {
        Self {
            family,
            producer,
            applicability,
            source,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn family_type(&self) -> TypeId {
        self.family
    }

    pub(in crate::domain_computation::primary_graph) fn producer_identity(&self) -> &str {
        &self.producer
    }

    pub(in crate::domain_computation::primary_graph) fn applicability(
        &self,
    ) -> WorthQueryProducerApplicability {
        self.applicability
    }

    pub(in crate::domain_computation::primary_graph) fn source_epoch(&self) -> &SourceEpoch {
        &self.source
    }

    pub(super) fn same_occurrence(&self, other: &Self) -> bool {
        self.family == other.family && self.source.same_occurrence(&other.source)
    }

    pub(super) fn same_semantic_source(&self, other: &Self) -> bool {
        // One installed binding may serve both lifecycle postures. Its Ready
        // keeps the same output meaning; delivery still certifies native
        // currentness. Another binding never joins through this predicate.
        self.family == other.family
            && self.producer == other.producer
            && self.applicability.profile_kind() == other.applicability.profile_kind()
            && self.source.same_semantic_source(&other.source)
    }

    pub(super) fn comparison_work(&self) -> Option<usize> {
        self.producer
            .len()
            .checked_add(self.applicability.profile_kind().len())?
            .checked_add(10)
    }

    pub(super) fn replacement_order(&self, other: &Self) -> Option<Ordering> {
        if self.family != other.family {
            return None;
        }
        let order = self.source.replacement_order(&other.source)?;
        if self.applicability.profile_kind() != other.applicability.profile_kind() {
            return None;
        }
        if self.producer == other.producer {
            return Some(order.then_with(|| {
                self.applicability
                    .lifecycle()
                    .cmp(&other.applicability.lifecycle())
            }));
        }
        // Another binding may succeed this row only through the declared
        // Initial→Preserve transition of the same profile, never text order.
        if self.applicability.profile_kind() != other.applicability.profile_kind()
            || self.applicability.lifecycle() == other.applicability.lifecycle()
        {
            return None;
        }
        Some(order.then_with(|| {
            self.applicability
                .lifecycle()
                .cmp(&other.applicability.lifecycle())
        }))
    }
}
