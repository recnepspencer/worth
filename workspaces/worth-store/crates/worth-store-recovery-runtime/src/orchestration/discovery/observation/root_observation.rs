use worth_store::physical_runtime::{BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalRecordFormatDeclaration,
    RootSelectorRole, ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{
    observe_structured_physical_root_candidate, PhysicalRootManifestDenial,
    PhysicalRootSelectorDenial, PhysicalRootSlotObservation,
};

use crate::entry::{
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimits, PhysicalRecoveryRootProtocolArtifact, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_addressed_root, admit_current_selector, admit_previous_selector,
    RecoveryArtifactNamespaceJoin, RecoveryIntegrityIngressRejection,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::counters::record_root_counters;
use super::map_selector_discovery_failure;
use crate::orchestration::discovery::{map_cumulative_discovery_failure, DiscoveryFailure};

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
    admitted: u64,
}

pub(super) fn observe_root_slots(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    expected_format: PhysicalRecordFormatDeclaration,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<RootObservations, DiscoveryFailure> {
    let declaration = limits.declaration();
    let current_source = read_current_selector(discovery)?;
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
            admitted: declaration.manifest_bytes,
        },
        counters,
    )?;
    let mut denials = current_denial.into_iter().collect::<Vec<_>>();
    let previous_source = read_previous_selector(discovery)
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
            admitted: declaration.manifest_bytes,
        },
        counters,
    )
    .map_err(|failure| failure.with_root_protocol_denials(&denials))?;
    record_root_counters(counters, &current, &previous);
    denials.extend(previous_denial);
    Ok(RootObservations {
        current,
        previous,
        remaining_manifest_bytes,
        denials,
    })
}

fn read_current_selector(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
) -> Result<ObservedRecoveryArtifact, DiscoveryFailure> {
    discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .map_err(map_selector_discovery_failure)
}

fn read_previous_selector(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
) -> Result<ObservedRecoveryArtifact, DiscoveryFailure> {
    discovery
        .read_previous_selector(ROOT_SELECTOR_BYTES as u64)
        .map_err(map_selector_discovery_failure)
}

fn observe_root_slot(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    scope: RootObservationScope,
    selector_source: ObservedRecoveryArtifact,
    budget: ManifestByteBudget<'_>,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<
    (
        PhysicalRootSlotObservation,
        Option<PhysicalRecoverySourceDenial>,
    ),
    DiscoveryFailure,
> {
    let selector = match admit_selector_source(scope, &selector_source, counters) {
        Ok(selector) => selector,
        Err(rejected) => return Ok(rejected),
    };
    let root = match read_and_admit_addressed_root(discovery, scope, selector, budget, counters)? {
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
    selector_source: &ObservedRecoveryArtifact,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<
    DurableRootSelector,
    (
        PhysicalRootSlotObservation,
        Option<PhysicalRecoverySourceDenial>,
    ),
> {
    let selector_artifact = selector_artifact(scope.role);
    let selector = match scope.role {
        RootSelectorRole::Current => admit_current_selector(
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
        RootSelectorRole::Previous => admit_previous_selector(
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
    let root_source = discovery
        .read_root_manifest(generation, *budget.remaining)
        .map_err(|failure| {
            map_cumulative_discovery_failure(
                failure,
                PhysicalRecoveryLimitDimension::ManifestEntries,
                PhysicalRecoveryLimitDimension::ManifestBytes,
                budget.admitted,
                *budget.remaining,
            )
        })?;
    let observed_bytes = root_source.bytes().map_or(0, |bytes| bytes.len() as u64);
    *budget.remaining = budget
        .remaining
        .checked_sub(observed_bytes)
        .ok_or(PhysicalRecoveryBlock::DiscoveryLimit)?;
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
        Err(rejection) => {
            return Ok(Err((
                PhysicalRootSlotObservation::RootRejected {
                    denial: root_denial(rejection),
                    selector,
                },
                Some(root_protocol_denial(root_artifact, rejection)),
            )));
        }
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
