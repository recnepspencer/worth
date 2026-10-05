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
    let WorthQueryCommittedComponentResolutionDenial::Unavailable(detail) = denial;
    recovery_evidence::unknown_commit_recovery_evidence(detail)
}

pub(super) enum WorthQueryCommittedComponentResolutionDenial {
    Unavailable(&'static str),
}

/// The caller resolving its own commit takes the World history hold its
/// publication handed over, so its receipt keeps `at_commit` readable for as
/// long as it lives; every other copy of the evidence stays detached.
pub(super) fn resolve_committed_components(
    context: &WorthQueryAuthorizedCompareContext<'_>,
    mut receipt: crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication,
) -> Result<WorthQueryResolvedCommitComponents, WorthQueryCommittedComponentResolutionDenial> {
    let causality =
        resolve_exact_committed_aftermath(context.aftermath_causality.as_ref(), &receipt).map_err(
            |_| {
                WorthQueryCommittedComponentResolutionDenial::Unavailable(
                    "committed aftermath causality could not be recovered",
                )
            },
        )?;
    context.provider.claim_fresh_history(&mut receipt);
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
