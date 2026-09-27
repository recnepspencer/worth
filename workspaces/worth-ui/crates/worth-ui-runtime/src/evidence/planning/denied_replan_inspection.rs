pub(crate) fn project_denied_replan_inspection(
    plan: &crate::runtime::UiNarrowedAllocationFramePlan,
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
    denial: &crate::runtime::UiAllocationReplanTransactionCommitDenial,
) -> worth_ui_inspection::UiAllocationInspectionDeniedAttempt {
    use worth_ui_inspection::{
        UiAllocationInspectionEvidenceFamily as EvidenceFamily,
        UiAllocationInspectionEvidenceRef as EvidenceRef,
        UiAllocationInspectionNeighborhoodIdentity as NeighborhoodIdentity,
        UiAllocationInspectionSelection,
    };
    let invalidation_identity = plan.identity().ingress_keys().iter().fold(
        0x776f7274682d696eu64 ^ plan.frame_epoch().as_u64(),
        |identity, key| {
            identity.rotate_left(7)
                ^ key.ingress_identity().as_u64()
                ^ key.source_generation().as_u64().rotate_left(17)
                ^ key.source_order().as_u64().rotate_left(31)
        },
    );
    let selection_identity = selection.ordered_neighborhoods().iter().fold(
        selection.primary().planning_identity_digest(),
        |identity, neighborhood| {
            identity.rotate_left(11) ^ neighborhood.identity().identity_digest()
        },
    );
    let reuse_denial = project_reuse_denial(denial);
    let denial_evidence = denial.evidence();
    worth_ui_inspection::UiAllocationInspectionDeniedAttempt::from_runtime_projection(
        plan.families()
            .iter()
            .copied()
            .map(super::project_stream_family)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        plan.narrowed_families()
            .map(super::project_invalidation_family)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        UiAllocationInspectionSelection::new(
            NeighborhoodIdentity::diagnostic(selection.primary().identity().identity_digest()),
            selection
                .ordered_neighborhoods()
                .iter()
                .map(|neighborhood| {
                    NeighborhoodIdentity::diagnostic(neighborhood.identity().identity_digest())
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            u16::try_from(
                selection
                    .ordered_neighborhoods()
                    .iter()
                    .filter(|neighborhood| neighborhood.widen_reason().is_some())
                    .count(),
            )
            .expect("a neighborhood selection counts its neighborhoods in u16"),
            EvidenceRef::diagnostic(
                EvidenceFamily::NeighborhoodSelectionArtifact,
                selection_identity,
            ),
        ),
        reuse_denial,
        project_denial_family(denial_evidence.family()),
        EvidenceRef::diagnostic(EvidenceFamily::InvalidationArtifact, invalidation_identity),
        EvidenceRef::diagnostic(
            EvidenceFamily::DenialArtifact,
            denial_evidence.identity().diagnostic_identity(),
        ),
    )
}

fn project_reuse_denial(
    denial: &crate::runtime::UiAllocationReplanTransactionCommitDenial,
) -> worth_ui_inspection::UiAllocationInspectionReuseDenialPosture {
    use crate::runtime::UiAllocationReuseDenial as Runtime;
    use worth_ui_inspection::UiAllocationInspectionReuseDenialPosture as Inspection;
    let crate::runtime::UiAllocationReplanTransactionCommitDenial::ReuseDenied { reason, .. } =
        denial
    else {
        return Inspection::NotApplicable;
    };
    match reason {
        Runtime::ReceiptIdentityMismatch => Inspection::ReceiptIdentityMismatch,
        Runtime::GenerationMismatch => Inspection::GenerationMismatch,
        Runtime::EquivalenceBasisMismatch => Inspection::EquivalenceBasisMismatch,
        Runtime::UnsupportedPartialReuse => Inspection::UnsupportedPartialReuse,
    }
}

fn project_denial_family(
    family: crate::runtime::UiAllocationDenialFamily,
) -> worth_ui_inspection::UiAllocationInspectionDenialFamily {
    use crate::runtime::UiAllocationDenialFamily as Runtime;
    use worth_ui_inspection::UiAllocationInspectionDenialFamily as Inspection;
    match family {
        Runtime::MissingSelection => Inspection::MissingSelection,
        Runtime::CandidateMismatch => Inspection::CandidateMismatch,
        Runtime::CandidatePlanning => Inspection::CandidatePlanning,
        Runtime::Reuse => Inspection::Reuse,
        Runtime::RecomputePending => Inspection::RecomputePending,
        Runtime::TransactionIdentity => Inspection::TransactionIdentity,
        Runtime::GenerationMismatch => Inspection::GenerationMismatch,
        Runtime::CommitBudget => Inspection::CommitBudget,
        Runtime::DurableMutationBudget => Inspection::DurableMutationBudget,
        Runtime::ResizeBasis => Inspection::ResizeBasis,
        Runtime::PortalAnchor => Inspection::PortalAnchor,
        Runtime::DurableSemanticState => Inspection::DurableSemanticState,
        Runtime::CatalogBinding => Inspection::CatalogBinding,
        Runtime::CounterExhaustion => Inspection::CounterExhaustion,
        Runtime::SourceSequence => Inspection::SourceSequence,
        Runtime::SourcePolicy => Inspection::SourcePolicy,
        Runtime::SourceAuthority => Inspection::SourceAuthority,
        Runtime::StaleHostEvidence => Inspection::StaleHostEvidence,
        Runtime::UnsupportedScrollOwnership => Inspection::UnsupportedScrollOwnership,
        Runtime::ContradictoryScrollOwnership => Inspection::ContradictoryScrollOwnership,
        Runtime::BrokenPortalAnchor => Inspection::BrokenPortalAnchor,
        Runtime::NeighborhoodLocality => Inspection::NeighborhoodLocality,
    }
}
