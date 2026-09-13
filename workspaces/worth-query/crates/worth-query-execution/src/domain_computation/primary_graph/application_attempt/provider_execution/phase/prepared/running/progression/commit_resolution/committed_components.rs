use super::*;

pub(super) struct WorthQueryResolvedCommitComponents {
    projection: WorthQueryCommittedReceiptProjection,
    causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    >,
}

pub(super) fn committed_component_recovery_evidence(
    denial: WorthQueryCommittedComponentResolutionDenial,
) -> WorthQueryApplicationUnresolvedCommitEvidence {
    match denial {
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            "committed aftermath recovery requires snapshot-capacity readmission",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable,
        ) => recovery_evidence::unknown_commit_recovery_evidence(
            "committed aftermath causality could not be recovered",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted,
            "committed aftermath recovery requires retention-capacity readmission",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted,
            "committed aftermath recovery exhausted retention identity space",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted,
            "committed aftermath recovery exhausted snapshot identity space",
        ),
        WorthQueryCommittedComponentResolutionDenial::Unavailable(detail) => {
            recovery_evidence::unknown_commit_recovery_evidence(detail)
        }
    }
}

pub(super) enum WorthQueryCommittedComponentResolutionDenial {
    Aftermath(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial),
    Unavailable(&'static str),
}

pub(super) fn resolve_committed_components(
    context: &WorthQueryAuthorizedCompareContext<'_>,
    receipt: crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication,
) -> Result<WorthQueryResolvedCommitComponents, WorthQueryCommittedComponentResolutionDenial> {
    let causality = resolve_exact_committed_aftermath(
        context.provider,
        context.aftermath_causality.as_ref(),
        &receipt,
    )
    .map_err(WorthQueryCommittedComponentResolutionDenial::Aftermath)?;
    let projection = WorthQueryCommittedReceiptProjection::resolve(receipt).map_err(|_| {
        WorthQueryCommittedComponentResolutionDenial::Unavailable(
            "committed dispatch outbox binding was denied",
        )
    })?;
    if projection
        .committed_dispatch_outbox()
        .map(|binding| binding.record())
        != context.dispatch_outbox.as_ref()
    {
        return Err(WorthQueryCommittedComponentResolutionDenial::Unavailable(
            "committed dispatch outbox evidence does not match the admitted attempt",
        ));
    }
    Ok(WorthQueryResolvedCommitComponents {
        projection,
        causality,
    })
}

pub(super) fn seal_committed_outcome(
    context: WorthQueryAuthorizedCompareContext<'_>,
    resolved: WorthQueryResolvedCommitComponents,
) -> WorthQueryProviderProgressionOutcome {
    let permit = WorthQueryFreshCommitReceiptPermit::mint(context.provider_session);
    let Some(receipt) = WorthQueryPendingApplicationCommitReceipt::from_projection(
        permit,
        resolved.projection,
        certify_provider_recomparison(context.preconditions),
        context.canonical_work,
        context.authority_binding,
    ) else {
        return WorthQueryProviderProgressionOutcome::Indeterminate(
            recovery_evidence::unknown_commit_recovery_evidence(
                "committed provider evidence belongs to another session",
            ),
        );
    };
    WorthQueryProviderProgressionOutcome::Committed(
        receipt.with_aftermath_causality(resolved.causality),
    )
}
