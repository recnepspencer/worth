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
        fact: &WorthQueryApplicationObservedFact,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, WorthQueryDecisionReadSetFailure> {
        super::check_control(control)?;
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
            return Ok(fact.remains_equal_in(runtime, snapshot));
        };
        // Preserve scalar malformed-limit refusal before index admission.
        if *candidate_limit == 0 || *candidate_limit == usize::MAX {
            return Ok(false);
        }
        let key = (*index_id, *entity_kind, locator.clone());
        if !self.selectors.contains_key(&key) {
            let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
                return Ok(false);
            };
            let Ok(prepared) = runtime.index_access().prepare_entity_field_lookup(
                &view,
                *index_id,
                *entity_kind,
                locator,
            ) else {
                return Ok(false);
            };
            self.selectors.insert(key.clone(), prepared);
        }
        let prepared = &self.selectors[&key];
        let current = runtime.index_access().execute_prepared_entity_field_lookup(
            prepared,
            value,
            *candidate_limit,
            BoundedIndexParityMode::Production,
            || super::check_control(control),
        );
        match current {
            Ok(current) => Ok(!current.overflowed()
                && current.retain_definition().as_ref() == definition.as_ref()
                && current.candidate_entity_ids() == candidates),
            Err(BoundedEntityFieldLookupAdmissionStop::Admission(denial)) => Err(denial),
            Err(_) => Ok(false),
        }
    }
}
