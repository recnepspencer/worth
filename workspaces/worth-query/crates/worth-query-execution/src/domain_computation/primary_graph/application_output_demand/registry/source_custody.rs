use super::{
    DemandState, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
    WorthQueryPerformedOutputDemandSource,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn bind_prepared_output_source(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        output_source_identity: [u8; 32],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((_, source)) = state
            .prepared_sources
            .iter_mut()
            .find(|(commit, _)| commit == source_commit)
        else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "prepared source is absent or has already been consumed",
            ));
        };
        source.output_source_identity = Some(output_source_identity);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn retain_performed_source(
        &self,
        performed_source: WorthQueryPerformedOutputDemandSource,
        preparation: &super::WorthQueryRequiredOutputSourcePreparation,
    ) -> Result<worth_runtime_world::facade::CompositeCommitIdentity, WorthQueryOutputDemandDenial>
    {
        let source_occurrence = performed_source.receipt.product_branch().occurrence();
        if preparation.occurrence != source_occurrence {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "required-output preparation belongs to another product occurrence",
            ));
        }
        let source_commit = performed_source.change.product_commit().clone();
        let source_identity = performed_source.source_identity;
        let source_scope = performed_source.receipt.principal_scope().scope();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .source_preparations
            .get(&source_occurrence)
            .is_none_or(|state| state.retired || state.active == 0)
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "product occurrence retired before required-output custody transfer",
            ));
        }
        if state
            .prepared_sources
            .iter()
            .any(|(commit, _)| commit == &source_commit)
            || state.records.values().any(|record| {
                record
                    .performed_source
                    .as_ref()
                    .is_some_and(|source| source.change.product_commit() == &source_commit)
            })
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is already retained by its output-demand owner",
            ));
        }
        if state.prepared_sources.iter().any(|(_, source)| {
            source.receipt.product_branch().occurrence() == source_occurrence
                && source.receipt.principal_scope().scope() == source_scope
                && WorthQueryOutputDemandKey::source_same_occurrence(
                    &source.source_identity,
                    &source_identity,
                )
                && WorthQueryOutputDemandKey::source_revision(&source.source_identity)
                    > WorthQueryOutputDemandKey::source_revision(&source_identity)
        }) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "a newer required-output source is already retained",
            ));
        }
        let mut superseded = Vec::new();
        state.prepared_sources.retain(|(commit, source)| {
            let older = source.receipt.product_branch().occurrence() == source_occurrence
                && source.receipt.principal_scope().scope() == source_scope
                && WorthQueryOutputDemandKey::source_same_occurrence(
                    &source.source_identity,
                    &source_identity,
                )
                && WorthQueryOutputDemandKey::source_revision(&source.source_identity)
                    < WorthQueryOutputDemandKey::source_revision(&source_identity);
            if older {
                superseded.push(commit.clone());
            }
            !older
        });
        for commit in superseded {
            state.retired_prepared_sources.insert(
                commit,
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "a newer required-output source replaced this custody",
                ),
            );
        }
        retire_stale_records(
            &mut state,
            source_occurrence,
            source_scope,
            &source_identity,
        );
        state
            .prepared_sources
            .push((source_commit.clone(), performed_source));
        Ok(source_commit)
    }
}

fn retire_stale_records(
    state: &mut super::DemandRegistryState,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    successor: &[u8; 32],
) {
    for (key, record) in &mut state.records {
        let stale = record.product_occurrence == occurrence
            && record.source_scope == Some(scope)
            && WorthQueryOutputDemandKey::source_same_occurrence(&key.source, successor)
            && key.revision() < WorthQueryOutputDemandKey::source_revision(successor);
        if stale {
            record.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                &key.producer,
            ));
            record.performed_source = None;
            record.wake.notify();
        }
    }
    state.records.retain(|key, record| {
        let stale = record.product_occurrence == occurrence
            && record.source_scope == Some(scope)
            && WorthQueryOutputDemandKey::source_same_occurrence(&key.source, successor)
            && key.revision() < WorthQueryOutputDemandKey::source_revision(successor);
        !stale || record.interests != 0
    });
}
