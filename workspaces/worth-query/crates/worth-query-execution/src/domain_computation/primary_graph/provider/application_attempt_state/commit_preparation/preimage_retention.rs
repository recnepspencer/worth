//! Fail-closed retention from the admitted decision read-set.

mod authored_anchors;
mod owned_relations;
mod relations;
pub(crate) mod selection;
pub(crate) use relations::WorthQueryRetainedRelationPreImageSeal;

pub(crate) use selection::WorthQueryRetainedPreImageSeal;

use crate::domain_computation::application_aftermath::{
    WorthQueryPreImageRetentionDenial, WorthQueryRetainedPreImage,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

use super::super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::provider::session_commit::provider_failure;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPreImageRetentionWork {
    validated_intents_examined: usize,
    mutation_targets_materialized: usize,
    decision_facts_examined: usize,
    candidates_materialized: usize,
    demanded_loci_examined: usize,
}

pub(super) struct WorthQueryPreparedPreImageRetention {
    retained: Option<WorthQueryRetainedPreImage>,
    work: WorthQueryPreImageRetentionWork,
}

impl WorthQueryPreparedPreImageRetention {
    pub(super) fn into_parts(
        self,
    ) -> (
        Option<WorthQueryRetainedPreImage>,
        WorthQueryPreImageRetentionWork,
    ) {
        (self.retained, self.work)
    }
}

pub(super) fn retain_attempt_preimage(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryPreparedPreImageRetention, WorthQueryProviderSessionFailure> {
    if let Some(source) = attempt
        .aftermath_causality()
        .map(|causality| causality.prepared_correction())
    {
        source.validate_relation_inverse(candidate, execution, layout)?;
    }
    let Some(demand) = attempt.preimage_demand() else {
        return Ok(WorthQueryPreparedPreImageRetention {
            retained: None,
            work: WorthQueryPreImageRetentionWork::default(),
        });
    };
    let footprint = candidate.mutation_footprint(Some(demand));
    let footprint_work = footprint.work();
    let footprint = footprint
        .into_projected()
        .ok_or_else(|| retention_failure(WorthQueryPreImageRetentionDenial::EmptyDemand))?;
    let candidates = attempt.facts().preimage_candidates(demand, &footprint, policy)
        .map_err(|cause| match cause {
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(denial) =>
                WorthQueryProviderSessionFailure::allocation_denied(denial, "authored preimage retention"),
            _ => provider_failure(WorthQueryProviderSessionProtocolStage::Commit,
                "preimage projection capacity exhausted"),
        })?;
    let examined = candidates.examined();
    let selected = if let Some(selection) = authored_anchors::select(attempt)? {
        selection::retain_from_authored_anchors(
            demand,
            candidates.iter(),
            &footprint,
            &selection.anchors,
            selection.own_changes,
        )
    } else {
        selection::retain_from_attempt(demand, candidates.iter(), &footprint)
    };
    let (mut retained, candidates_materialized) = selected.map_err(retention_failure)?.into_parts();
    if !demand.relations().is_empty() {
        retained = relations::retain(
            attempt,
            execution,
            layout,
            demand,
            candidate,
            candidates.as_slice(),
            retained,
        )?;
    }
    retained
        .retain_content(
            execution,
            attempt
                .affinity()
                .product_publication()
                .observation()
                .basis()
                .relational_basis(),
            demand.maximum_encoded_bytes(),
        )
        .map_err(retention_failure)?;
    Ok(WorthQueryPreparedPreImageRetention {
        retained: Some(retained),
        work: WorthQueryPreImageRetentionWork {
            validated_intents_examined: footprint_work.validated_intents_examined(),
            mutation_targets_materialized: footprint_work.mutation_targets_materialized(),
            decision_facts_examined: examined,
            candidates_materialized,
            demanded_loci_examined: demand.loci().len() + demand.relations().len(),
        },
    })
}

impl WorthQueryPreImageRetentionWork {
    pub(in crate::domain_computation::primary_graph) const fn validated_intents_examined(
        self,
    ) -> usize {
        self.validated_intents_examined
    }

    pub(in crate::domain_computation::primary_graph) const fn mutation_targets_materialized(
        self,
    ) -> usize {
        self.mutation_targets_materialized
    }

    pub(in crate::domain_computation::primary_graph) const fn decision_facts_examined(
        self,
    ) -> usize {
        self.decision_facts_examined
    }

    pub(in crate::domain_computation::primary_graph) const fn candidates_materialized(
        self,
    ) -> usize {
        self.candidates_materialized
    }

    pub(in crate::domain_computation::primary_graph) const fn demanded_loci_examined(
        self,
    ) -> usize {
        self.demanded_loci_examined
    }
}

fn retention_failure(
    denial: WorthQueryPreImageRetentionDenial,
) -> WorthQueryProviderSessionFailure {
    let semantic =
        |detail| provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail);
    let failure = match denial {
        WorthQueryPreImageRetentionDenial::Content(cause) => {
            let stop = crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(
                cause.into_execution_cause(), None);
            WorthQueryProviderSessionFailure::checked_read_stopped(
                stop,
                WorthQueryProviderSessionProtocolStage::Commit,
                "preimage content admission stopped before the Native transaction",
            )
        }
        WorthQueryPreImageRetentionDenial::MissingDemandedField => {
            semantic("recorded inverse demands an unobserved exact mutated field")
        }
        WorthQueryPreImageRetentionDenial::ExceedsByteBound => {
            semantic("retained pre-image exceeds the installed demand byte bound")
        }
        WorthQueryPreImageRetentionDenial::EmptyDemand => {
            semantic("installed recorded inverse declares an empty pre-image demand")
        }
        WorthQueryPreImageRetentionDenial::AmbiguousDemandedField => semantic(
            "recorded inverse has several admitted observations for one exact mutated field",
        ),
        WorthQueryPreImageRetentionDenial::NoMutatedRecord => {
            semantic("recorded inverse has no existing mutated field to retain")
        }
    };
    failure.with_preimage_retention_denial(denial)
}

#[cfg(test)]
mod tests;
