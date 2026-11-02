//! Admitting a performed source and its required output obligation.
use super::*;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_performed(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)?
            .unwrap_or_else(|| requested_key.clone());
        let join = state.prepare_performed_join(
            &requested_key,
            &key,
            source_commit,
            source_scope,
            product_occurrence,
        )?;
        let required_member = state.prepare_required_member(&key)?;
        let prepared_record = if state.records.contains_key(&key) {
            None
        } else {
            Some(state.prepare_new_record(
                &key,
                Some(source_commit),
                product_occurrence,
                source_scope,
                None,
            )?)
        };
        supersede_predecessors(&mut state, &key)?;
        if let Some(prepared) = prepared_record {
            state.install_prepared_record(prepared);
        }
        state.install_performed_join(&key, join);
        let mut record = state.records.get_mut(&key).expect("admitted row exists");
        record.interests = record.interests.saturating_add(1);
        record.required_interests += 1;
        let interest = interest(self, key, &record, true);
        record.finish();
        state.install_required_member(required_member);
        Ok(interest)
    }
}
