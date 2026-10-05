use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::data::error::SignalError;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::tests::domains::fintech::world::{
    FinancialAspect, FinancialLocalityAction, FinancialLocalityAdmissionPolicy,
    FinancialLocalityComparisonPolicy, FinancialLocalityDefinition, FinancialLocalityOutputPolicy,
    LocalityEconomicOwner, LocalityScope,
};

use super::super::locality_expectation::{
    ExpectedSealedOriginBinding, FinancialLocalityExpectationManifest,
};

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("WORTH.signal.financial-certification-case");

pub(super) fn identity_entries(
    definition: &FinancialLocalityDefinition,
    manifest: &FinancialLocalityExpectationManifest,
    tier: DiagnosticsTier,
) -> Result<Vec<CanonicalBasisEntry>, SignalError> {
    let trace = definition
        .action_traces()
        .iter()
        .find(|trace| trace.identity() == manifest.action_trace())
        .expect("manifest trace identity belongs to the immutable locality definition");
    let mut entries = vec![
        text("diagnostics.tier", tier_token(tier)),
        text("execution.posture", posture_token(trace.actions())),
        text("mutation.rule", "signal.locality-action-trace.v1"),
        text("policy.rule", "signal.locality-policy.v1"),
        text("work.rule", "signal.locality-expected-work.v1"),
        text("counter.rule", "signal.invalidation.semantic.v2"),
    ];
    entries.push(digest_tokens(
        "mutation.actions",
        trace.actions().iter().map(action_token),
    )?);
    entries.push(digest_tokens(
        "policy.outputs",
        definition.outputs().iter().map(|output| {
            format!(
                "{}:{}",
                output.id.ordinal(),
                policy_token(output.execution_policy())
            )
        }),
    )?);
    entries.push(digest_tokens(
        "expected.work",
        manifest.canonical_work().iter().map(|(work, origins)| {
            format!(
                "target={};revision={};epoch={};stage={};origins={}",
                work.target.ordinal(),
                work.dependency_revision,
                work.readiness_epoch,
                work.stage_order,
                origins
                    .iter()
                    .map(origin_token)
                    .collect::<Vec<_>>()
                    .join("|")
            )
        }),
    )?);
    entries.push(digest_tokens(
        "expected.counters",
        super::super::ExpectedLocalityCounterRow::ALL
            .into_iter()
            .filter(|row| !row.is_physical_batch_shape())
            .map(|row| format!("{}={}", row as u8, manifest.counter_manifest().value(row))),
    )?);
    Ok(entries)
}

fn digest_tokens(
    locus: &'static str,
    tokens: impl IntoIterator<Item = String>,
) -> Result<CanonicalBasisEntry, SignalError> {
    const CHUNK_SIZE: usize = 1_024;
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    for (chunk_ordinal, chunk) in tokens.chunks(CHUNK_SIZE).enumerate() {
        let entries = chunk.iter().enumerate().map(|(item_ordinal, token)| {
            CanonicalBasisEntry::new(
                DOMAIN,
                CanonicalBasisLocus::Named(
                    format!("{locus}.chunk.{chunk_ordinal}.item.{item_ordinal}").into(),
                ),
                CanonicalBasisEntryKind::Identity,
                CanonicalBasisValue::ExactText(token.clone().into()),
            )
        });
        chunks.push(derive_digest(entries)?);
    }
    let digest = if chunks.len() == 1 {
        chunks[0]
    } else {
        derive_digest(chunks.into_iter().enumerate().map(|(ordinal, digest)| {
            CanonicalBasisEntry::new(
                DOMAIN,
                CanonicalBasisLocus::Named(format!("{locus}.chunk.{ordinal}").into()),
                CanonicalBasisEntryKind::Identity,
                CanonicalBasisValue::BytesDigest(digest),
            )
        }))?
    };
    Ok(CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(digest),
    ))
}

fn derive_digest(
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> Result<CanonicalDigestId, SignalError> {
    let version = CanonicalizationRuleVersion::new("1").expect("locality basis rule is valid");
    let ready = match prepare_canonical_basis_sequence(version, DOMAIN, entries) {
        TransitionOutcome::Success(ready) => ready,
        denied => {
            return Err(SignalError::internal(format!(
                "locality identity axis was denied: {denied:?}"
            )))
        }
    };
    let ready = match canonicalization()
        .digest()
        .for_sequence(ready, CanonicalDigestAlgorithmId::sha256())
    {
        TransitionOutcome::Success(ready) => ready,
        denied => {
            return Err(SignalError::internal(format!(
                "locality identity axis digest was denied: {denied:?}"
            )))
        }
    };
    let digest = canonicalization().digest().derive(ready);
    Ok(CanonicalDigestId::new(*digest.value().bytes()))
}

fn action_token(action: &FinancialLocalityAction) -> String {
    match action {
        FinancialLocalityAction::CommitFactor(value) => format!(
            "commit:{}:{}:{}:{}:{}",
            value.producer.ordinal(),
            aspect_token(value.aspect),
            scope_token(value.scope),
            value.admission_generation,
            value.publication_order
        ),
        FinancialLocalityAction::RetryAdmission {
            target,
            retry_ordinal,
        } => format!("retry:{}:{retry_ordinal}", target.ordinal()),
        FinancialLocalityAction::StagePreRewireWork { round, binding } => format!(
            "stage-rewire:{round}:{}:{}:{}",
            binding.target.ordinal(),
            binding.dependency_revision,
            binding.readiness_epoch
        ),
        FinancialLocalityAction::StageSourceRecompute { obligation } => format!(
            "stage-source:{}:{}:{}:{}:{}",
            obligation.source.ordinal(),
            aspect_token(obligation.aspect),
            scope_token(obligation.scope),
            obligation.admission_generation,
            obligation.dependency_revision
        ),
        FinancialLocalityAction::AcceptedOwnerMove { round, change } => format!(
            "owner-move:{round}:{}:{}:{}:{}:{}:{}:{}",
            change.target.ordinal(),
            owner_token(change.before_owner),
            owner_token(change.after_owner),
            subscription_token(change.before_subscription),
            subscription_token(change.after_subscription),
            change.structural.topology_mutation_ordinal,
            change.structural.resulting_dependency_revision
        ),
        FinancialLocalityAction::RejectStaleWork {
            round,
            stale,
            current_dependency_revision,
        } => format!(
            "reject-stale:{round}:{}:{}:{}:{current_dependency_revision}",
            stale.target.ordinal(),
            stale.dependency_revision,
            stale.readiness_epoch
        ),
        FinancialLocalityAction::AcceptedDependencyRemoval {
            round,
            owner,
            removed_subscription,
            structural,
        } => format!(
            "remove:{round}:{}:{}:{}:{}",
            owner_token(*owner),
            subscription_token(*removed_subscription),
            structural.topology_mutation_ordinal,
            structural.resulting_dependency_revision
        ),
        FinancialLocalityAction::AcceptedDependencyRecreation {
            round,
            owner,
            subscription,
            structural,
        } => format!(
            "recreate:{round}:{}:{}:{}:{}",
            owner_token(*owner),
            subscription_token(*subscription),
            structural.topology_mutation_ordinal,
            structural.resulting_dependency_revision
        ),
        FinancialLocalityAction::RejectedCycle {
            round,
            target,
            attempted_subscription,
            attempted_topology_ordinal,
            retained_dependency_revision,
        } => format!(
            "cycle:{round}:{}:{}:{attempted_topology_ordinal}:{retained_dependency_revision}",
            target.ordinal(),
            subscription_token(*attempted_subscription)
        ),
        FinancialLocalityAction::CaptureBranch { branch_ordinal } => {
            format!("branch:{branch_ordinal}")
        }
        FinancialLocalityAction::CaptureCheckpoint { checkpoint_ordinal } => {
            format!("checkpoint:{checkpoint_ordinal}")
        }
        FinancialLocalityAction::DestroyDerivedState {
            destruction_ordinal,
        } => format!("destroy:{destruction_ordinal}"),
        FinancialLocalityAction::ReadmitFreshRuntime { runtime_epoch } => {
            format!("readmit:{runtime_epoch}")
        }
        FinancialLocalityAction::ReplayCanonicalTrace { replay_ordinal } => {
            format!("replay:{replay_ordinal}")
        }
        FinancialLocalityAction::DeterministicRerun { rerun_ordinal } => {
            format!("rerun:{rerun_ordinal}")
        }
    }
}

fn policy_token(
    policy: crate::tests::domains::fintech::world::FinancialLocalityExecutionPolicy,
) -> String {
    let admission = match policy.admission {
        FinancialLocalityAdmissionPolicy::Always => "always".to_owned(),
        FinancialLocalityAdmissionPolicy::ChangedSubscribedAspect(aspects) => format!(
            "changed:{}",
            aspects
                .into_iter()
                .map(aspect_token)
                .collect::<Vec<_>>()
                .join("+")
        ),
    };
    let comparison = match policy.dependency_comparison {
        FinancialLocalityComparisonPolicy::ExactEconomicRevision => "exact-revision",
    };
    let output = match policy.output_equivalence {
        FinancialLocalityOutputPolicy::ExactEconomicRevision => "exact-revision",
    };
    format!("admission={admission};dependency={comparison};output={output}")
}

fn origin_token(origin: &ExpectedSealedOriginBinding) -> String {
    match origin {
        ExpectedSealedOriginBinding::SourceRecompute {
            admission_generation,
        } => format!("source:{admission_generation}"),
        ExpectedSealedOriginBinding::DependencyCommit {
            cause_set_generation,
            producer_commit_ordinals,
        } => format!(
            "dependency:{cause_set_generation}:{}",
            producer_commit_ordinals
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join("+")
        ),
        ExpectedSealedOriginBinding::StructuralRecompute {
            structural_generation,
        } => format!("structural:{structural_generation}"),
    }
}

fn posture_token(actions: &[FinancialLocalityAction]) -> &'static str {
    if actions
        .iter()
        .any(|action| matches!(action, FinancialLocalityAction::ReplayCanonicalTrace { .. }))
    {
        "replay-derived"
    } else if actions
        .iter()
        .any(|action| matches!(action, FinancialLocalityAction::ReadmitFreshRuntime { .. }))
    {
        "restored"
    } else {
        "warm"
    }
}

fn tier_token(tier: DiagnosticsTier) -> &'static str {
    match tier {
        DiagnosticsTier::Operational => "operational",
        DiagnosticsTier::Development => "development",
        DiagnosticsTier::Forensic => "forensic",
    }
}

fn aspect_token(aspect: FinancialAspect) -> &'static str {
    match aspect {
        FinancialAspect::Price => "price",
        FinancialAspect::Curve => "curve",
        FinancialAspect::Volatility => "volatility",
        FinancialAspect::Risk => "risk",
        FinancialAspect::Alert => "alert",
    }
}

fn scope_token(scope: Option<LocalityScope>) -> String {
    match scope {
        None => "all".to_owned(),
        Some(scope) => format!(
            "{}:{}",
            scope.region,
            scope.detail.map_or("*".to_owned(), |v| v.to_string())
        ),
    }
}

fn subscription_token(
    subscription: crate::tests::domains::fintech::world::FinancialLocalitySubscription,
) -> String {
    format!(
        "{}:{}:{}:{}",
        subscription.upstream.ordinal(),
        aspect_token(subscription.input_aspect),
        scope_token(subscription.edge_scope),
        scope_token(subscription.eligibility_scope)
    )
}

fn owner_token(owner: LocalityEconomicOwner) -> String {
    match owner {
        LocalityEconomicOwner::MarketDataFeed(value) => format!("feed-{value}"),
        LocalityEconomicOwner::Position(value) => format!("position-{value}"),
        LocalityEconomicOwner::BookRisk(value) => format!("book-{value}"),
        LocalityEconomicOwner::DeskRisk(value) => format!("desk-{value}"),
        LocalityEconomicOwner::AuditControl(value) => format!("audit-{value}"),
        LocalityEconomicOwner::RegulatoryReport(value) => format!("report-{value}"),
    }
}

fn text(locus: &'static str, value: &'static str) -> CanonicalBasisEntry {
    owned_text(locus.to_owned(), value.to_owned())
}

fn owned_text(locus: String, value: String) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::ExactText(value.into()),
    )
}
