//! Admitting a performed source and its required output obligation.

use super::obligation_capacity::prepare_performed_obligation_slots;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_performed(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(custody) = state.source_custody.get(source_commit) else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is absent or has already been consumed",
            ));
        };
        if let Some(denial) = &custody.retired {
            return Err(denial.clone());
        }
        if let Some(denial) = custody.source_denial(&requested_key.source) {
            return Err(denial);
        }
        if !custody.available(&requested_key.source, source_scope) {
            if custody.bound_sources.is_some() {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "performed source identity does not match its prepared publication",
                ));
            }
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is absent or has already been consumed",
            ));
        }
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)
            .unwrap_or_else(|| requested_key.clone());
        admission.charge_external_work(4).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "required work activation exceeds request work",
            )
        })?;
        let required_member = state.prepare_required_member(&key)?;
        state.charge_record_lookup(&key, admission)?;
        if let Some(record) = state.records.get(&key) {
            if record.product_occurrence != product_occurrence
                || record.source_scope != Some(source_scope)
            {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "matching output identity belongs to another source scope or occurrence",
                ));
            }
            if let DemandState::Failed(denial) = &record.state {
                return Err(denial.clone());
            }
        }
        state.charge_record_lookup(&key, admission)?;
        let (prepared_obligations, new_bytes) = prepare_performed_obligation_slots(&state, &key)?;
        state.charge_record_lookup(&key, admission)?;
        let prepared_record = if state.records.contains_key(&key) {
            None
        } else {
            Some(state.prepare_new_record(
                &key,
                Some(source_commit),
                product_occurrence,
                source_scope,
                None,
                admission,
            )?)
        };
        state.charge_record_lookup(&key, admission)?;
        let prepared_commit_growth = state
            .records
            .get(&key)
            .and_then(|record| (!record.source_commits.contains(source_commit)).then_some(record))
            .map(|record| state.prepare_source_commit_growth(record, admission))
            .transpose()?
            .flatten();
        if prepared_record.is_some() {
            state.charge_record_lookup_after_insert(&key, admission)?;
        } else {
            state.charge_record_lookup(&key, admission)?;
        }
        supersede_predecessors(&mut state, &requested_key)?;
        let custody = state
            .source_custody
            .get_mut(source_commit)
            .expect("selected custody exists");
        let mut performed_source = custody
            .source
            .as_ref()
            .expect("available custody retains source")
            .clone();
        performed_source.output_source_identity = Some(requested_key.source.clone());
        custody.finish_admission(requested_key.source.clone());
        if let Some(prepared) = prepared_record {
            state.install_prepared_record(prepared);
        }
        let record = state.records.get_mut(&key).expect("admitted record exists");
        if !record.source_commits.contains(source_commit) {
            if let Some(growth) = prepared_commit_growth {
                growth.install(record);
            }
            record.source_commits.push(source_commit.clone());
        }
        let previous = std::mem::replace(&mut record.performed_obligations, prepared_obligations);
        let old_bytes = previous
            .capacity()
            .saturating_mul(std::mem::size_of::<super::super::PerformedOutputObligation>());
        record.performed_obligations.extend(previous);
        record
            .performed_obligations
            .push(super::super::PerformedOutputObligation {
                source_commit: source_commit.clone(),
                source: requested_key.source.clone(),
            });
        if matches!(record.state, DemandState::Admitted) && record.performed_source.is_none() {
            record.performed_source = Some(performed_source);
        } else {
            drop(performed_source);
        }
        record.interests = record.interests.saturating_add(1);
        record.required_interests += 1;
        let interest = interest(self, key, record, true);
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_add(new_bytes)
            .saturating_sub(old_bytes);
        state.install_required_member(required_member);
        Ok(interest)
    }
}
