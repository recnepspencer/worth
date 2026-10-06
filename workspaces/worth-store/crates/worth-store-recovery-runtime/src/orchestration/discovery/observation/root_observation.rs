use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, PhysicalRecoveryReadAllocation,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalRecordFormatDeclaration,
    RootSelectorRole,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{
    observe_structured_physical_root_candidate, PhysicalRootManifestDenial,
    PhysicalRootSelectorDenial, PhysicalRootSlotObservation,
};

use crate::entry::{
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock,
    PhysicalRecoveryLimitDimension as Dimension, PhysicalRecoveryLimits,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_addressed_root, admit_current_selector, admit_previous_selector,
    RecoveryArtifactNamespaceJoin, RecoveryIntegrityIngressRejection,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::counters::record_root_counters;
use crate::orchestration::discovery::DiscoveryFailure;
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};
use crate::orchestration::recovery_budget::RecoveryAllowance;

mod funded_read;
pub(super) use funded_read::window_admission_failure;
use funded_read::FundedRootReads;

pub(super) struct RootObservations {
    pub(super) current: PhysicalRootSlotObservation,
    pub(super) previous: PhysicalRootSlotObservation,
    pub(super) remaining_manifest_bytes: u64,
    pub(super) denials: Vec<PhysicalRecoverySourceDenial>,
}

#[derive(Clone, Copy)]
struct RootObservationScope {
    role: RootSelectorRole,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
}

struct ManifestByteBudget<'a> {
    remaining: &'a mut u64,
    whole: RecoveryAllowance,
}

pub(super) fn observe_root_slots(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    expected_format: PhysicalRecordFormatDeclaration,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    allocation: &mut PhysicalRecoveryReadAllocation<'_>,
) -> Result<RootObservations, DiscoveryFailure> {
    let declaration = limits.declaration();
    let mut reads = FundedRootReads::new(allocation, declaration);
    let current_source = reads.read_selector(discovery, RootSelectorRole::Current)?;
    let store = discovery.store_identity();
    let mut remaining_manifest_bytes = declaration.manifest_bytes;
    let (current, current_denial) = observe_root_slot(
        discovery,
        RootObservationScope {
            role: RootSelectorRole::Current,
            store,
            format: expected_format,
        },
        current_source,
        ManifestByteBudget {
            remaining: &mut remaining_manifest_bytes,
            whole: RecoveryAllowance::declared(&declaration, Dimension::ManifestBytes),
        },
        counters,
        &mut reads,
    )?;
    reads.finish_slot();
    let mut denials = current_denial.into_iter().collect::<Vec<_>>();
    let previous_source = reads
        .read_selector(discovery, RootSelectorRole::Previous)
        .map_err(|failure| failure.with_root_protocol_denials(&denials))?;
    counters.selector_slots = discovery.counters().fixed_slots_read;
    let (previous, previous_denial) = observe_root_slot(
        discovery,
        RootObservationScope {
            role: RootSelectorRole::Previous,
            store,
            format: expected_format,
        },
        previous_source,
        ManifestByteBudget {
            remaining: &mut remaining_manifest_bytes,
            whole: RecoveryAllowance::declared(&declaration, Dimension::ManifestBytes),
        },
        counters,
        &mut reads,
    )
    .map_err(|failure| failure.with_root_protocol_denials(&denials))?;
    reads.finish_slot();
    record_root_counters(counters, &current, &previous);
    denials.extend(previous_denial);
    Ok(RootObservations {
        current,
        previous,
        remaining_manifest_bytes,
        denials,
    })
}

fn observe_root_slot(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    scope: RootObservationScope,
    selector_source: Result<ObservedRecoveryArtifact, OversizedArtifact>,
    budget: ManifestByteBudget<'_>,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    reads: &mut FundedRootReads<'_, '_>,
) -> Result<
    (
        PhysicalRootSlotObservation,
        Option<PhysicalRecoverySourceDenial>,
    ),
    DiscoveryFailure,
> {
    let selector = match admit_selector_source(scope, selector_source.as_ref(), counters) {
        Ok(selector) => selector,
        Err(rejected) => return Ok(rejected),
    };
    let root =
        match read_and_admit_addressed_root(discovery, scope, selector, budget, counters, reads)? {
            Ok(root) => root,
            Err(rejected) => return Ok(rejected),
        };
    match scope.role {
        RootSelectorRole::Current => counters.current_root_candidate_interpretations += 1,
        RootSelectorRole::Previous => counters.previous_root_candidate_interpretations += 1,
    }
    Ok((
        observe_structured_physical_root_candidate(selector, root.0, root.1),
        None,
    ))
}

fn admit_selector_source(
    scope: RootObservationScope,
    selector_source: Result<&ObservedRecoveryArtifact, &OversizedArtifact>,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<
    DurableRootSelector,
    (
        PhysicalRootSlotObservation,
        Option<PhysicalRecoverySourceDenial>,
    ),
> {
    let selector_artifact = selector_artifact(scope.role);
    let selector = match (selector_source, scope.role) {
        // A selector is its fixed frame and nothing more.
        (Err(OversizedArtifact), _) => Err(RecoveryIntegrityIngressRejection::NonCanonicalEncoding),
        (Ok(selector_source), RootSelectorRole::Current) => admit_current_selector(
            RecoveryArtifactNamespaceJoin::from_canonical(selector_source),
            scope.store,
            scope.format,
        )
        .map(|admitted| {
            counters.current_selector_integrity_admissions += 1;
            let selector = admitted.project();
            counters.current_selector_interpretations += 1;
            selector
        }),
        (Ok(selector_source), RootSelectorRole::Previous) => admit_previous_selector(
            RecoveryArtifactNamespaceJoin::from_canonical(selector_source),
            scope.store,
            scope.format,
        )
        .map(|admitted| {
            counters.previous_selector_integrity_admissions += 1;
            let selector = admitted.project();
            counters.previous_selector_interpretations += 1;
            selector
        }),
    };
    let selector = match selector {
        Ok(selector) => selector,
        Err(RecoveryIntegrityIngressRejection::Absent) => {
            return Err((
                PhysicalRootSlotObservation::Absent,
                Some(root_protocol_denial(
                    selector_artifact,
                    RecoveryIntegrityIngressRejection::Absent,
                )),
            ));
        }
        Err(rejection) => {
            let denial = selector_denial(rejection);
            return Err((
                PhysicalRootSlotObservation::SelectorRejected(denial),
                Some(root_protocol_denial(selector_artifact, rejection)),
            ));
        }
    };
    Ok(selector)
}

fn read_and_admit_addressed_root(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    scope: RootObservationScope,
    selector: DurableRootSelector,
    budget: ManifestByteBudget<'_>,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    reads: &mut FundedRootReads<'_, '_>,
) -> Result<
    Result<
        (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration),
        (
            PhysicalRootSlotObservation,
            Option<PhysicalRecoverySourceDenial>,
        ),
    >,
    DiscoveryFailure,
> {
    let generation = selector.root_generation();
    let root_artifact = root_artifact(scope.role, generation);
    let rejected = |rejection| {
        (
            PhysicalRootSlotObservation::RootRejected {
                denial: root_denial(rejection),
                selector,
            },
            Some(root_protocol_denial(root_artifact, rejection)),
        )
    };
    // A root manifest is one page of the format its selector declares.
    let ceiling = ReadCeiling::within(
        u64::from(selector.format().page_size().bytes()),
        budget.whole.admitted(),
        *budget.remaining,
    );
    let root_source = match reads.read_root(discovery, scope.role, generation, ceiling)? {
        Ok(root_source) => root_source,
        Err(OversizedArtifact) => {
            return Ok(Err(rejected(
                RecoveryIntegrityIngressRejection::NonCanonicalEncoding,
            )));
        }
    };
    let observed_bytes = root_source.bytes().map_or(0, |bytes| bytes.len() as u64);
    // The read was asked for no more than was left. T2: a read grant charges
    // what it returns.
    *budget.remaining = budget
        .remaining
        .checked_sub(observed_bytes)
        .ok_or_else(|| {
            super::super::refused_beside(
                budget.whole,
                observed_bytes,
                *budget.remaining,
                PhysicalRecoveryBlock::MediaObservation,
            )
        })?;
    if root_source.bytes().is_some() {
        reads.reserve_canonical_scratch(root_artifact)?;
    }
    let (root, root_format) = match admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&root_source),
        scope.store,
        selector.format(),
        generation,
    ) {
        Ok(admitted) => {
            match scope.role {
                RootSelectorRole::Current => counters.current_root_integrity_admissions += 1,
                RootSelectorRole::Previous => counters.previous_root_integrity_admissions += 1,
            }
            admitted.project()
        }
        Err(rejection) => return Ok(Err(rejected(rejection))),
    };
    Ok(Ok((root, root_format)))
}

fn selector_artifact(role: RootSelectorRole) -> PhysicalRecoveryRootProtocolArtifact {
    match role {
        RootSelectorRole::Current => PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
        RootSelectorRole::Previous => PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
    }
}

fn root_artifact(role: RootSelectorRole, generation: u64) -> PhysicalRecoveryRootProtocolArtifact {
    match role {
        RootSelectorRole::Current => {
            PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation }
        }
        RootSelectorRole::Previous => {
            PhysicalRecoveryRootProtocolArtifact::PreviousRoot { generation }
        }
    }
}

fn selector_denial(rejection: RecoveryIntegrityIngressRejection) -> PhysicalRootSelectorDenial {
    match rejection {
        RecoveryIntegrityIngressRejection::ConflictingDuplication { .. } => {
            PhysicalRootSelectorDenial::Conflict
        }
        RecoveryIntegrityIngressRejection::Integrity(PhysicalIntegrityRejection::Damaged(
            localization,
        )) if matches!(
            localization.cause(),
            PhysicalDamageCause::StoreIdentityMismatch
                | PhysicalDamageCause::SelectorRoleMismatch
                | PhysicalDamageCause::FormatMismatch
        ) =>
        {
            PhysicalRootSelectorDenial::AuthorityMismatch
        }
        _ => PhysicalRootSelectorDenial::Integrity,
    }
}

fn root_denial(rejection: RecoveryIntegrityIngressRejection) -> PhysicalRootManifestDenial {
    match rejection {
        RecoveryIntegrityIngressRejection::ConflictingDuplication { .. } => {
            PhysicalRootManifestDenial::Conflict
        }
        _ => PhysicalRootManifestDenial::Integrity,
    }
}

fn root_protocol_denial(
    artifact: PhysicalRecoveryRootProtocolArtifact,
    rejection: RecoveryIntegrityIngressRejection,
) -> PhysicalRecoverySourceDenial {
    PhysicalRecoverySourceDenial::RootProtocol {
        artifact,
        denial: rejection.diagnostic(),
    }
}
