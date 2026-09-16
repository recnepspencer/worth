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
        let Some(source_index) = state
            .prepared_sources
            .iter()
            .position(|(commit, _)| commit == source_commit)
        else {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "prepared source is absent or has already been consumed",
            ));
        };
        let source = &state.prepared_sources[source_index].1;
        if source.output_source_identity.is_some() {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "prepared source identity is already bound",
            ));
        }
        let source_occurrence = source.receipt.product_branch().occurrence();
        let source_scope = source.receipt.principal_scope().scope();
        if state
            .prepared_sources
            .iter()
            .enumerate()
            .any(|(index, (_, candidate))| {
                index != source_index
                    && candidate.receipt.product_branch().occurrence() == source_occurrence
                    && candidate.receipt.principal_scope().scope() == source_scope
                    && candidate.output_source_identity.is_some_and(|bound| {
                        WorthQueryOutputDemandKey::source_same_occurrence(
                            &bound,
                            &output_source_identity,
                        ) && WorthQueryOutputDemandKey::source_revision(&bound)
                            >= WorthQueryOutputDemandKey::source_revision(&output_source_identity)
                    })
            })
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                "an equal or newer required-output source is already retained",
            ));
        }

        let mut superseded = Vec::new();
        let mut retained = Vec::with_capacity(state.prepared_sources.len());
        for (index, prepared) in std::mem::take(&mut state.prepared_sources)
            .into_iter()
            .enumerate()
        {
            let older = index != source_index
                && prepared.1.receipt.product_branch().occurrence() == source_occurrence
                && prepared.1.receipt.principal_scope().scope() == source_scope
                && prepared.1.output_source_identity.is_some_and(|bound| {
                    WorthQueryOutputDemandKey::source_same_occurrence(
                        &bound,
                        &output_source_identity,
                    ) && WorthQueryOutputDemandKey::source_revision(&bound)
                        < WorthQueryOutputDemandKey::source_revision(&output_source_identity)
                });
            if older {
                superseded.push(prepared.0);
            } else {
                retained.push(prepared);
            }
        }
        state.prepared_sources = retained;
        for commit in superseded {
            state.retired_prepared_sources.insert(
                commit,
                denial(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "a newer required-output source replaced this custody",
                ),
            );
        }
        retire_stale_records(
            &mut state,
            source_occurrence,
            source_scope,
            &output_source_identity,
        );
        state
            .prepared_sources
            .iter_mut()
            .find(|(commit, _)| commit == source_commit)
            .expect("the bound custody was retained above")
            .1
            .output_source_identity = Some(output_source_identity);
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
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "required-output preparation belongs to another product occurrence",
            ));
        }
        let source_commit = performed_source.change.product_commit().clone();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .source_preparations
            .get(&source_occurrence)
            .is_none_or(|preparation| preparation.retired || preparation.active == 0)
        {
            return Err(denial(
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
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is already retained by its output-demand owner",
            ));
        }
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
            record.state = DemandState::Failed(denial(
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

fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
