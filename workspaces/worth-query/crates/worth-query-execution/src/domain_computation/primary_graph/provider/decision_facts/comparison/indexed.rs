//! Lazy selector admission at the comparison owner's single exact root.
use crate::domain_computation::{
    primary_graph::application_attempt::{
        retained_decision_facts::StorageControl, WorthQueryApplicationObservedFact,
    },
    WorthQueryDecisionReadSetFailure,
};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::{
    identity::KindId,
    indexes::{
        BoundedEntityFieldLookupAdmissionStop, BoundedIndexParityMode, DerivedIndexId,
        PreparedEntityFieldLookup,
    },
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

// This directory is call-local and descriptive; its map backing is not charged
// as ExecutionArray payload. Every value is the native owner's exact-root token.
pub(super) struct PreparedSelections {
    pub(super) selectors:
        BTreeMap<(DerivedIndexId, KindId, AspectFieldLocator), PreparedEntityFieldLookup>,
}
impl PreparedSelections {
    pub(super) fn new() -> Self {
        Self {
            selectors: BTreeMap::new(),
        }
    }
    pub(super) fn clear(&mut self) {
        self.selectors.clear();
    }

    pub(super) fn remains_equal(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fact: &WorthQueryApplicationObservedFact,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, WorthQueryDecisionReadSetFailure> {
        super::check_control(control)?;
        execution.check_live().map_err(|stop| super::super::fact_read_set_failure(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::request_read_stopped(stop, "prepared indexed comparison")))?;
        let WorthQueryApplicationObservedFact::IndexedEntitySelection {
            index_id,
            definition,
            entity_kind,
            locator,
            value,
            candidate_limit,
            candidates,
        } = fact
        else {
            return fact
                .remains_equal_in(runtime, execution, snapshot)
                .map_err(super::super::fact_read_set_failure);
        };
        // Preserve scalar malformed-limit refusal before index admission.
        if *candidate_limit == 0 || *candidate_limit == usize::MAX {
            return Ok(false);
        }
        let key = (*index_id, *entity_kind, locator.clone());
        if !self.selectors.contains_key(&key) {
            let Some(view) = runtime.read_truth().project_snapshot(snapshot, execution) else {
                return Ok(false);
            };
            let prepared = match runtime.index_access().prepare_entity_field_lookup(
                &view, *index_id, *entity_kind, locator, execution,
            ) {
                Ok(prepared) => prepared,
                Err(denial) => match denial.kind() {
                    worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause) => {
                        return Err(super::super::fact_read_set_failure(
                            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "prepared indexed admission"),
                        ));
                    }
                    worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::IndexNotInstalled
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::WrongIndexKind
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::StorageParityMismatch => return Ok(false),
                },
            };
            self.selectors.insert(key.clone(), prepared);
        }
        let prepared = &self.selectors[&key];
        let current = runtime.index_access().execute_prepared_entity_field_lookup(
            prepared,
            value,
            *candidate_limit,
            BoundedIndexParityMode::Production,
            execution,
            || super::check_control(control),
        );
        match current {
            Ok(current) => Ok(!current.overflowed()
                && current.retain_definition().as_ref() == definition.as_ref()
                && current.candidate_entity_ids() == candidates),
            Err(BoundedEntityFieldLookupAdmissionStop::Admission(denial)) => Err(denial),
            Err(BoundedEntityFieldLookupAdmissionStop::Lookup(denial)) => match denial.kind() {
                worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause) => Err(super::super::fact_read_set_failure(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "prepared indexed comparison"))),
                worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::IndexNotInstalled
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::WrongIndexKind
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
 | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::StorageParityMismatch => Ok(false),
            },
            Err(BoundedEntityFieldLookupAdmissionStop::AccountingOverflow) => Err(super::super::fact_read_set_failure(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(worth_relational::facade::transactions::RelationalExecutionDenialCause::MemoryOverflow, "prepared indexed quote"))),
            Err(BoundedEntityFieldLookupAdmissionStop::ExactBasisRequired) => Ok(false),
        }
    }
}
