//! Exact producer rows and installed-family lifecycle succession.

use super::SourceEpoch;

use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability;
use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;
use std::cmp::Ordering;

// Observe semantic row comparisons, not the ordered navigation bought by a
// path charge. The two explicit equality gates record their comparisons too.
#[cfg(test)]
thread_local! { static ROW_COMPARISONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

#[cfg(test)]
pub(super) fn observe_row_comparison() {
    ROW_COMPARISONS
        .with(|count| count.set(count.get().checked_add(1).expect("row comparison counter")));
}

// Family and source precede the binding: one output occurrence remains a
// contiguous range even when Initial and Preserve have separate executors.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandKey {
    pub(super) family: OutputFamilyIdentity,
    pub(super) source: SourceEpoch,
    pub(super) applicability: WorthQueryProducerApplicability,
    pub(super) producer: String,
}

impl WorthQueryOutputDemandKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        family: &'static str,
        producer: String,
        applicability: WorthQueryProducerApplicability,
        source: SourceEpoch,
    ) -> Self {
        Self {
            family: OutputFamilyIdentity::declared(family),
            producer,
            applicability,
            source,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn family_identity(&self) -> &str {
        self.family.as_str()
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

    pub(super) fn replacement_order(&self, other: &Self) -> Option<Ordering> {
        #[cfg(test)]
        observe_row_comparison();
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
        // Initial-to-Preserve transition of the same profile, never text order.
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
