use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_performed_source(
        &self,
        source: WorthQueryPerformedOutputDemandSource,
        preparation: &super::super::WorthQueryRequiredOutputSourcePreparation,
        root_kind: PreparedOutputRootKind,
        discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<CompositeCommitIdentity, WorthQueryOutputDemandDenial> {
        let occurrence = source.receipt.product_branch().occurrence();
        if preparation.occurrence != occurrence {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "preparation belongs to another product occurrence",
            ));
        }
        let commit = source.change.product_commit().clone();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .source_preparations
            .get(&occurrence)
            .is_none_or(|preparation| preparation.retired || preparation.active == 0)
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Closed,
                "product occurrence retired before custody transfer",
            ));
        }
        if super::super::post_commit::custody_reader::visible(&state, &commit) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is already retained",
            ));
        }
        super::super::post_commit::custody_reader::prune_invisible_slot(&mut state, &commit);
        let scope = source.receipt.principal_scope().scope();
        let direct_root = matches!(root_kind, PreparedOutputRootKind::Required(_));
        if direct_root
            && state.source_custody.iter().any(|(prior, candidate)| {
                prior.ordinal() >= commit.ordinal()
                    && candidate.retired.is_none()
                    && candidate.root_kind == root_kind
                    && candidate.bound_sources.is_none()
                    && candidate.source.as_ref().is_some_and(|retained| {
                        retained.receipt.product_branch().occurrence() == occurrence
                            && retained.receipt.principal_scope().scope() == scope
                    })
            })
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                "a newer source publication owns this preparation scope",
            ));
        }
        // Admit the abandoned-source scratch before retiring any prior custody.
        // The ordered custody index itself has no capacity reservation.
        let mut abandoned = Vec::new();
        if matches!(root_kind, PreparedOutputRootKind::Discovered(_)) {
            for (prior, _) in state
                .source_custody
                .iter()
                .filter(|(prior, candidate)| {
                    prior.ordinal() < commit.ordinal()
                        && candidate.retired.is_none()
                        && candidate.root_kind == root_kind
                        && candidate.occurrence == occurrence
                        && candidate.token_count == 0
                        && candidate.source.as_ref().is_some_and(|retained| {
                            retained.receipt.principal_scope().scope() == scope
                        })
                })
                .filter(|(prior, candidate)| {
                    !candidate.bound_sources.iter().flatten().any(|source| {
                        state
                            .records
                            .source_occurrence_rows(&source.identity)
                            .any(|(_, record)| {
                                record.source_commits.contains(*prior) && record.interests != 0
                            })
                    })
                })
            {
                abandoned
                    .try_reserve(super::reservation_control::reservation_request(&state))
                    .map_err(|_| source_capacity_denial())?;
                abandoned.push(prior.clone());
            }
        }
        state.note_custody_changes(|prior, candidate| {
            if direct_root
                && prior.ordinal() < commit.ordinal()
                && candidate.retired.is_none()
                && candidate.root_kind == root_kind
                && candidate.bound_sources.is_none()
                && candidate.source.as_ref().is_some_and(|retained| {
                    retained.receipt.product_branch().occurrence() == occurrence
                        && retained.receipt.principal_scope().scope() == scope
                })
            {
                candidate.retire(denial(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "a newer source publication replaced this preparation",
                ));
                return true;
            }
            false
        });
        if matches!(root_kind, PreparedOutputRootKind::Discovered(_)) {
            for prior in abandoned {
                if let Some(candidate) = state.source_custody.get_mut(&prior) {
                    candidate.retire(denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "a newer discovered publication replaced abandoned source custody",
                    ));
                }
                state.note_custody(&prior);
            }
        }
        let recovery = discovery.map(|discovery| super::super::DiscoveredSourceRecovery {
            receipt: source.receipt.clone(),
            observation: source.observation.clone(),
            discovery,
        });
        state.source_custody.insert(
            commit.clone(),
            SourceCustody {
                occurrence,
                root_kind,
                source: Some(source),
                discovery: recovery,
                bound_sources: None,
                consumed_sources: Vec::new(),
                retired_sources: Vec::new(),
                retired: None,
                token_count: 1,
                cleanup: Default::default(),
                completed: false,
            },
        );
        Ok(commit)
    }

    pub(in crate::domain_computation::primary_graph) fn recover_discovered_source<
        Discovery: Clone + Send + Sync + 'static,
    >(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        root_type: TypeId,
    ) -> Result<
        (
            Discovery,
            worth_runtime_world::facade::ProductBranchObservation,
        ),
        WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let visible = super::super::post_commit::custody_reader::visible(
            &state,
            receipt.committed_product_publication().composite_commit(),
        );
        let custody = state
            .source_custody
            .get_mut(receipt.committed_product_publication().composite_commit())
            .filter(|_| visible)
            .filter(|custody| custody.retired.is_none())
            .filter(|custody| custody.root_kind == PreparedOutputRootKind::Discovered(root_type))
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "exact discovered source is not retained",
                )
            })?;
        let recovery = custody
            .discovery
            .as_ref()
            .filter(|retained| retained.receipt.same_retained_output_source_as(receipt))
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "exact discovered program source is not retained",
                )
            })?;
        let discovery = recovery
            .discovery
            .downcast_ref::<Discovery>()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "discovered root has a different typed query",
                )
            })?
            .clone();
        let observation = recovery.observation.clone();
        custody.token_count += 1;
        Ok((discovery, observation))
    }
}

fn source_capacity_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded,
        "performed source custody storage cannot be allocated",
    )
}
