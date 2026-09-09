use crate::domain_computation::{
    WorthQueryBoundInvariantExecutionView, WorthQueryInvariantExecutionFailure,
    WorthQueryInvariantStateLocator,
};

use super::super::application_attempt_state::WorthQueryStagedApplicationAttempt;

pub(super) struct ApplicationInvariantCandidateMaterial {
    pub(super) semantic: ApplicationInvariantSemanticMaterial,
    pub(super) batch: worth_relational::facade::transactions::WorkerIntentBatch,
    pub(super) branch: worth_relational::facade::history::BranchId,
    pub(super) product:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    pub(super) decision_facts: usize,
    pub(super) aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    pub(super) application_graph_reads:
        worth_query_installation::facade::WorthQueryOperationGraphReadContract,
    pub(super) application_touches:
        worth_query_installation::facade::WorthQueryOperationTouchContract,
    pub(super) application_read_touch_overlap:
        worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex,
}

pub(super) struct ApplicationInvariantSemanticMaterial {
    pub(super) expected: Vec<WorthQueryInvariantStateLocator>,
    load_evidence: String,
}

impl ApplicationInvariantCandidateMaterial {
    pub(super) fn from_staged(
        staged: &WorthQueryStagedApplicationAttempt<'_>,
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        Ok(Self {
            semantic: ApplicationInvariantSemanticMaterial::from_staged(staged)?,
            batch: staged.batch().clone(),
            branch: staged.branch().clone(),
            product: staged.product_publication().clone(),
            decision_facts: staged.decision_fact_count(),
            aftermath_causality: staged.aftermath_causality().cloned(),
            application_graph_reads: staged
                .application_graph_reads()
                .cloned()
                .ok_or_else(super::touch_failure)?,
            application_touches: staged
                .application_touches()
                .cloned()
                .ok_or_else(super::touch_failure)?,
            application_read_touch_overlap: staged
                .application_read_touch_overlap()
                .cloned()
                .ok_or_else(super::touch_failure)?,
        })
    }
}

impl ApplicationInvariantSemanticMaterial {
    pub(super) fn from_staged(
        staged: &WorthQueryStagedApplicationAttempt<'_>,
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        Ok(Self {
            expected: expected_locators(staged.overlay_facts())?,
            load_evidence: format!("primary-invariant-load:{}", staged.overlay_identity()),
        })
    }

    pub(super) fn validate_load(
        &self,
        execution: &WorthQueryBoundInvariantExecutionView<'_>,
        evidence: crate::domain_computation::WorthQueryInvariantStateLoadEvidenceView<'_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        (execution.state_load_plan().locators() == self.expected.as_slice()
            && evidence.loaded_fact_locators() == self.expected.as_slice()
            && evidence.physical_load_evidence() == self.load_evidence)
            .then_some(())
            .ok_or_else(|| {
                super::closure_failure(
                    "invariant execution evidence differs from the exact proposed state",
                )
            })
    }
}

fn expected_locators(
    facts: &[crate::domain_computation::WorthQueryProposedFact],
) -> Result<Vec<WorthQueryInvariantStateLocator>, WorthQueryInvariantExecutionFailure> {
    let mut expected = facts
        .iter()
        .map(|fact| {
            WorthQueryInvariantStateLocator::new("application-proposed-state", fact.identity())
        })
        .collect::<Result<Vec<_>, _>>()?;
    expected.sort();
    let original_len = expected.len();
    expected.dedup();
    if expected.len() != original_len {
        return Err(super::closure_failure(
            "proposed effects contain duplicate invariant fact identities",
        ));
    }
    Ok(expected)
}
