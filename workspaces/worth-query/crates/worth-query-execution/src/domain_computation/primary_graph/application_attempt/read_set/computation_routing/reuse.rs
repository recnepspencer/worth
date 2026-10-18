//! Fixed-shape completion consumes the comparator's original native custody.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
    application_contribution::CompletedComputationRetention,
    fact_verification_observation as observation,
};
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;
use worth_relational::facade::{identity::EntityId, snapshots::SnapshotHandle};

pub(in crate::domain_computation::primary_graph) struct NativeReadRefresh {
    prior: SealedComputationFacts,
    layout: Arc<CheckedReadLayout>,
    // Shared metadata drops before its original settlement's actual funding.
    _custody:
        crate::domain_computation::primary_graph::application_contribution::NativeComputationReads,
}

impl ComputationFactAttribution {
    pub(in crate::domain_computation::primary_graph) fn native_refresh(
        &self,
        completed: &CompletedComputationRetention,
        contracts: &Arc<Contracts>,
        root: EntityId,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<NativeReadRefresh>, StoreDenial> {
        control.check_live()?;
        let Some(native) = &self.native else {
            return Ok(None);
        };
        if !native.at_snapshot(snapshot) {
            observation::native_completion_decline("snapshot");
            return Ok(None);
        }
        let ranges = self.range_closed.as_ref();
        let closes = match ranges {
            Some(ranges) => {
                ranges.closes(native, &self.fresh_calls)
                    && completed.closes_native_ranges(native, ranges)
            }
            None => completed.closes_native_reads(native, &self.carried),
        };
        if !closes {
            observation::native_completion_decline("completed-carried-calls");
            return Ok(None);
        }
        let prior = native.facts();
        let Some(layout) = prior.layout.checked() else {
            observation::native_completion_decline("checked-layout");
            return Ok(None);
        };
        if !Arc::ptr_eq(&layout.contracts, contracts) {
            observation::native_completion_decline("compiled-contract-backing");
            return Ok(None);
        }
        if !layout.scopes.same_contract(contracts) {
            observation::native_completion_decline("scope-contract-backing");
            return Ok(None);
        }
        if layout.root != root {
            observation::native_completion_decline("scope-root");
            return Ok(None);
        }
        if ranges.is_none()
            && !self
                .calls
                .iter()
                .inspect(|_| observation::completion_call_roster())
                .eq(layout.calls.iter())
        {
            observation::native_completion_decline("call-roster");
            return Ok(None);
        }
        if ranges.is_none() && !self.carried.is_disjoint(&self.fresh_calls) {
            observation::native_completion_decline("fresh-carried-overlap");
            return Ok(None);
        }
        if self
            .handler
            .as_ref()
            .is_none_or(|handler| !handler.iter().eq(layout.handlers.keys()))
        {
            observation::native_completion_decline("handler-keys");
            return Ok(None);
        }
        // This scratch index contains ONLY newly executed reads. Its values
        // borrow keys; no fact/key payload is copied and no retained index is
        // reconstructed. The full call roster above remains measured work.
        let mut quote = Some(0_u64);
        for reads in self.readers.values() {
            observation::completion_fresh_key();
            let count = usize::from(reads.membership)
                .checked_add(reads.item_keys.len())
                .and_then(|count| count.checked_add(reads.partitions.len()))
                .and_then(|count| u64::try_from(count).ok());
            let per = (11 * size_of::<(ComputationRead, Vec<&WorthQueryApplicationFactKey>)>()
                + 16 * size_of::<usize>()
                + 4 * size_of::<&WorthQueryApplicationFactKey>()) as u64;
            quote = quote
                .zip(count)
                .and_then(|(sum, count)| sum.checked_add(per.checked_mul(count)?));
        }
        let funding = control
            .policy()
            .reserve_memory(quote.ok_or(StoreDenial::Representability)?);
        control.check_live()?;
        let _funding = funding?;
        let mut fresh: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (key, reads) in &self.readers {
            control.check_live()?;
            observation::completion_fresh_key();
            for read in reads.reads() {
                fresh.entry(read).or_default().push(key);
            }
        }
        for read in &self.fresh_calls {
            control.check_live()?;
            if !prior
                .index
                .keys(*read)
                .eq(fresh.get(read).into_iter().flatten().copied())
            {
                observation::native_completion_decline("fresh-call-keys");
                return Ok(None);
            }
        }
        Ok(Some(NativeReadRefresh {
            prior: prior.clone(),
            layout: Arc::clone(layout),
            _custody: native.clone(),
        }))
    }
}

impl NativeReadRefresh {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn position(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Option<super::super::InstalledScopePosition> {
        self.layout.scopes.position(self.ordinal(key)?)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn decisions(
        &self,
    ) -> FactPayload {
        self.prior.original.slice(0..self.layout.decision_count)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn ordinal(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Option<usize> {
        self.prior
            .metadata
            .get(key)
            .map(|fact| fact.ordinal)
            .or_else(|| self.layout.handlers.get(key).copied())
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn fact(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Option<&super::super::super::WorthQueryApplicationObservedFact> {
        self.prior.original.get(self.ordinal(key)?)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn content(
        &self,
    ) -> Option<super::super::super::FactContent> {
        self.prior
            .content
            .clone()
            .filter(|content| content.same_body(&self.prior.original))
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn finish(
        mut self,
        original: FactPayload,
        content: Option<super::super::super::FactContent>,
    ) -> (
        SealedComputationFacts,
        crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        Arc<[usize]>,
        usize,
    ) {
        let old_content = self
            .prior
            .content
            .as_ref()
            .map_or(Some(0), |value| value.charged_bytes());
        let new_content = content
            .as_ref()
            .map_or(Some(0), |value| value.charged_bytes());
        self.prior.charged = self
            .prior
            .charged
            .zip(self.prior.original.retained_resident_bytes())
            .and_then(|(total, old)| total.checked_sub(old))
            .zip(original.retained_resident_bytes())
            .and_then(|(metadata, body)| metadata.checked_add(body))
            .zip(old_content)
            .and_then(|(total, old)| total.checked_sub(old))
            .zip(new_content)
            .and_then(|(total, new)| total.checked_add(new));
        self.prior.original = original;
        self.prior.content = content;
        (
            self.prior,
            self.layout.scopes.clone(),
            Arc::clone(&self.layout.computation_ordinals),
            self.layout.decision_count,
        )
    }
}
