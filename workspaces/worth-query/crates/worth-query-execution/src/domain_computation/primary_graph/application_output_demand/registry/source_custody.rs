use super::{
    BoundOutputSource, DemandRegistryState, DemandState, PreparedOutputRootKind, SourceCustody,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
    WorthQueryPerformedOutputDemandSource,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use std::cmp::Ordering;
use worth_runtime_world::facade::CompositeCommitIdentity;

mod retention;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn validate_recovery_root_kind(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        kind: PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state
            .source_custody
            .get(receipt.committed_product_publication().composite_commit())
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "the requested publication has no retained output custody",
                )
            })?;
        if let Some(retired) = &custody.retired {
            return Err(retired.clone());
        }
        if custody.root_kind != kind
            || custody
                .source
                .as_ref()
                .is_none_or(|source| !source.receipt.same_retained_output_source_as(receipt))
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "recovery root kind or receipt differs from retained source custody",
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn retained_prepared_source_observation(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        root_type: std::any::TypeId,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state
            .source_custody
            .get_mut(receipt.committed_product_publication().composite_commit())
            .filter(|custody| custody.retired.is_none())
            .filter(|custody| custody.root_kind == PreparedOutputRootKind::Required(root_type))
            .filter(|custody| {
                custody
                    .source
                    .as_ref()
                    .is_some_and(|source| source.receipt.same_retained_output_source_as(receipt))
            })?;
        custody.token_count += 1;
        custody
            .source
            .as_ref()
            .map(|source| source.observation.clone())
    }

    pub(in crate::domain_computation::primary_graph) fn ensure_prepared_output_source_bound(
        &self,
        commit: &CompositeCommitIdentity,
        source: BoundOutputSource,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.ensure_bound(commit, &[source])
    }

    pub(in crate::domain_computation::primary_graph) fn validate_prepared_recovery_currentness(
        &self,
        commit: &CompositeCommitIdentity,
        retained: [u8; 32],
        current: [u8; 32],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if retained == current {
            return Ok(());
        }
        if WorthQueryOutputDemandKey::source_same_occurrence(&retained, &current)
            && WorthQueryOutputDemandKey::source_revision(&current)
                > WorthQueryOutputDemandKey::source_revision(&retained)
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let denial = denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                "a newer source revision retired this prepared output root",
            );
            if let Some(custody) = state.source_custody.get_mut(commit) {
                if matches!(custody.root_kind, PreparedOutputRootKind::Required(_)) {
                    custody.retire(denial.clone());
                } else if custody.source_denial(&retained).is_none() {
                    custody.retired_sources.push((retained, denial.clone()));
                }
            }
            return Err(denial);
        }
        Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "current recovery source differs from the retained source occurrence",
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_prepared_output_source(
        &self,
        commit: &CompositeCommitIdentity,
        source: BoundOutputSource,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.bind_prepared_output_sources(commit, &[source])
    }

    pub(in crate::domain_computation::primary_graph) fn bind_prepared_output_sources(
        &self,
        commit: &CompositeCommitIdentity,
        sources: &[BoundOutputSource],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if sources.iter().enumerate().any(|(index, source)| {
            sources[..index].iter().any(|prior| {
                WorthQueryOutputDemandKey::source_same_occurrence(&prior.identity, &source.identity)
            })
        }) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "root discovery must contain distinct output source occurrences",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state.source_custody.get(commit).ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "prepared source is absent",
            )
        })?;
        if let Some(retired) = &custody.retired {
            return Err(retired.clone());
        }
        if custody.bound_sources.is_some() || custody.source.is_none() {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "prepared root batch was already bound",
            ));
        }
        let source = custody
            .source
            .as_ref()
            .expect("active custody retains source");
        let occurrence = source.receipt.product_branch().occurrence();
        let superseded = sources
            .iter()
            .filter(|source| {
                state.source_custody.iter().any(|(other_commit, other)| {
                    other_commit != commit
                        && other.retired.is_none()
                        && other.occurrence == occurrence
                        && other.bound_sources.as_ref().is_some_and(|bound| {
                            bound.iter().any(|prior| {
                                other.source_denial(&prior.identity).is_none()
                                    && root_revision_order(prior, source) == Some(Ordering::Greater)
                            })
                        })
                })
            })
            .map(|source| source.identity)
            .collect::<Vec<_>>();
        for (other_commit, other) in &mut state.source_custody {
            if other_commit == commit || other.retired.is_some() {
                continue;
            }
            if other.occurrence == occurrence {
                other.retire_superseded_roots(sources);
            }
        }
        for source in sources {
            retire_stale_records(&mut state, occurrence, source.scope, &source.identity);
        }
        let mut bound = sources.to_vec();
        bound.sort_by_key(|source| source.identity);
        let custody = state
            .source_custody
            .get_mut(commit)
            .expect("selected custody retained");
        for identity in superseded {
            if custody.source_denial(&identity).is_none() {
                custody.retired_sources.push((
                    identity,
                    denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "a newer required-output source is retained for this root",
                    ),
                ));
            }
        }
        custody.bound_sources = Some(bound);
        state.prune_completed_custody();
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn ensure_discovered_sources_bound(
        &self,
        commit: &CompositeCommitIdentity,
        sources: &[BoundOutputSource],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.ensure_bound(commit, sources)
    }

    fn ensure_bound(
        &self,
        commit: &CompositeCommitIdentity,
        sources: &[BoundOutputSource],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let bound = || {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state
                .source_custody
                .get(commit)
                .map(|custody| (custody.retired.clone(), custody.bound_sources.clone()))
        };
        let mut expected = sources.to_vec();
        expected.sort_by_key(|source| source.identity);
        if let Some((Some(retired), _)) = bound() {
            return Err(retired);
        }
        if let Some((_, Some(existing))) = bound() {
            return if existing == expected {
                Ok(())
            } else {
                Err(denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "rediscovery differs from the bound root source set",
                ))
            };
        }
        match self.bind_prepared_output_sources(commit, sources) {
            Ok(()) => Ok(()),
            Err(_)
                if bound().is_some_and(|(retired, bound)| {
                    retired.is_none() && bound == Some(expected)
                }) =>
            {
                Ok(())
            }
            Err(denial) => Err(denial),
        }
    }
}

impl SourceCustody {
    pub(super) fn retire(&mut self, cause: WorthQueryOutputDemandDenial) {
        self.source = None;
        self.discovery = None;
        self.retired = Some(cause);
    }

    pub(super) fn retire_superseded_roots(&mut self, successors: &[BoundOutputSource]) {
        let retired = self
            .bound_sources
            .as_ref()
            .into_iter()
            .flatten()
            .filter(|prior| {
                successors
                    .iter()
                    .any(|successor| root_revision_order(prior, successor) == Some(Ordering::Less))
            })
            .map(|prior| prior.identity)
            .collect::<Vec<_>>();
        for identity in retired {
            if self.source_denial(&identity).is_none() {
                self.retired_sources.push((
                    identity,
                    denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "a newer required-output source replaced this root",
                    ),
                ));
            }
        }
    }
}

pub(super) fn root_revision_order(
    prior: &BoundOutputSource,
    successor: &BoundOutputSource,
) -> Option<Ordering> {
    (prior.scope == successor.scope
        && WorthQueryOutputDemandKey::source_same_occurrence(&prior.identity, &successor.identity))
    .then(|| {
        WorthQueryOutputDemandKey::source_revision(&prior.identity).cmp(
            &WorthQueryOutputDemandKey::source_revision(&successor.identity),
        )
    })
}

fn retire_stale_records(
    state: &mut DemandRegistryState,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    successor: &[u8; 32],
) {
    for (key, record) in &mut state.records {
        if record.product_occurrence == occurrence
            && record.source_scope == Some(scope)
            && WorthQueryOutputDemandKey::source_same_occurrence(&key.source, successor)
            && key.revision() < WorthQueryOutputDemandKey::source_revision(successor)
        {
            let cause = denial(WorthQueryOutputDemandDenialKind::Superseded, &key.producer);
            match &mut record.state {
                DemandState::Output(output) => output.stop(cause),
                _ => record.state = DemandState::Failed(cause),
            }
            record.performed_source = None;
            record.wake.notify();
        }
    }
    state.records.retain(|key, record| {
        record.product_occurrence != occurrence
            || record.source_scope != Some(scope)
            || !WorthQueryOutputDemandKey::source_same_occurrence(&key.source, successor)
            || key.revision() >= WorthQueryOutputDemandKey::source_revision(successor)
            || record.interests != 0
    });
}

fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
