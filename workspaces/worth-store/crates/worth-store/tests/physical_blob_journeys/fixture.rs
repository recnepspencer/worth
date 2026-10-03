use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_foundational::{
    aspects, AspectValue, ContractValidationInput, InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store::aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalAuthorityWitness, StorePhysicalBoundaryWitness,
    ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalDurabilityPolicy, AdmittedPhysicalRecordFormat,
    AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy, CheckpointMemoryLimit,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalDurabilityStateReopenFailure, PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordPlacementPolicy, PhysicalRuntimeAdmission, PhysicalSignalConstructionFailure,
    PhysicalStore, PhysicalWalPolicy, RecordBootstrapDenial, RecordBootstrapFailure,
    RecordServingAdmissionOutcome, RecordStoreInitializationDenial, RecordStoreOpenDenial,
    RetainedWalTailLimit, ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_authority::require_current_store_authority;
use worth_store_contracts::ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY;
use worth_store_physical_backend::FilesystemAccessPosture;
use worth_store_security::{
    admit_store_security_scope, StoreAuthenticityRequirement, StoreAuthenticityRequirementClass,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};

pub(super) fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    (format, placement, access)
}

pub(super) fn placement() -> AdmittedRecordPlacementPolicy {
    configuration().1
}

pub(super) fn serving_from_initialization(root: &Path) -> ServingPhysicalRuntime {
    let (_, placement, _) = configuration();
    serving_from_initialization_with_placement(root, placement)
}

pub(super) fn serving_from_initialization_with_placement(
    root: &Path,
    placement: AdmittedRecordPlacementPolicy,
) -> ServingPhysicalRuntime {
    serving_from_initialization_with_placement_and_tail(root, placement, 64 * 1024 * 1024)
}

pub(super) fn serving_from_initialization_with_format_and_placement(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
) -> ServingPhysicalRuntime {
    let media = media(root);
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let durability = durability(&media);
    success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

pub(super) fn serving_from_initialization_with_placement_and_tail(
    root: &Path,
    placement: AdmittedRecordPlacementPolicy,
    retained_wal_tail_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_limits(&media, 16 * 1024 * 1024, retained_wal_tail_bytes);
    success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

pub(super) fn serving_from_initialization_with_wal_segment_bytes(
    root: &Path,
    segment_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability_with_wal_segment_bytes(&media, segment_bytes);
    success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

pub(super) fn serving_from_open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(media.open_record_store(PhysicalRecordOpen::new(format, access, durability)))
}

pub(super) fn assert_released_open_requires_recovered_custody(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
) {
    let media = media(root);
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let durability = durability(&media);
    let TransitionOutcome::Denied(denial) = media
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    else {
        panic!("selected release-head root must deny Serving without C8 custody")
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
    );
    denial.into_runtime().close();
}

/// A tier-anchored root whose retained WAL refuses its verified tier custody
/// fails ordinary durability reopen before Serving: only C8 may hand it off.
pub(super) fn assert_anchored_open_requires_recovered_custody(root: &Path) {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let TransitionOutcome::Failed(inspection) = media
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    else {
        panic!("an anchored root without admissible clean custody must not serve")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::RecoveredCheckpointCustodyRequired
                )
            )
        ),
        "unexpected reopen cause: {cause:?}"
    );
}

pub(super) fn serving_from_open_with_retained_wal_tail(
    root: &Path,
    retained_wal_tail_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_limits(&media, 16 * 1024 * 1024, retained_wal_tail_bytes);
    success(media.open_record_store(PhysicalRecordOpen::new(format, access, durability)))
}

pub(super) fn serving_from_open_with_wal_segment_bytes(
    root: &Path,
    segment_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_wal_segment_bytes(&media, segment_bytes);
    success(media.open_record_store(PhysicalRecordOpen::new(format, access, durability)))
}

pub(super) fn admitted_blob_scope(identity_key: &str) -> AdmittedBlobScope {
    let witness = StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();
    admitted_blob_scope_with_witness(identity_key, witness)
}

pub(super) fn admitted_blob_scope_for_replay_boundary(identity_key: &str) -> AdmittedBlobScope {
    let witness = StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary_instance(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
            ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY,
        )
        .unwrap(),
    )
    .unwrap();
    admitted_blob_scope_with_witness(identity_key, witness)
}

fn admitted_blob_scope_with_witness(
    identity_key: &str,
    witness: StorePhysicalBoundaryWitness,
) -> AdmittedBlobScope {
    let key = aspects().vocabulary().key(identity_key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String);
    let validated =
        match aspects()
            .validate()
            .against(&contract)
            .value(ContractValidationInput::from(AspectValue::String(
                InternedString::from("blob-authority"),
            ))) {
            TransitionOutcome::Success(value) => value,
            outcome => panic!("blob authority value must validate: {outcome:?}"),
        };
    let state = match aspects().authoritative_state().admit([validated]) {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("blob authority state must admit: {outcome:?}"),
    };
    let boundary = StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key),
        StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let authority = require_current_store_authority(boundary);
    let key_scope = StoreKeyScope::BlobChunkEnvelope;
    let tenant_scope = StoreTenantScope::TenantPhysicalBoundary;
    let authenticity = StoreAuthenticityRequirement::required(
        StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
    );
    let custody = StoreCustodyPosture::InternalStoreCustody;
    let expectation =
        StoreSecurityScopeAdmissionExpectation::new(key_scope, tenant_scope, authenticity, custody);
    let request = StoreSecurityScopeAdmissionRequest::new(
        &authority,
        key_scope,
        StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    );
    let admitted = match admit_store_security_scope(request) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("blob custody scope must admit: {outcome:?}"),
    };
    AdmittedBlobScope::from_store_security(admitted).unwrap()
}

fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("blob journey media must admit"),
    }
}

fn durability(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_wal_segment_bytes(media, 16 * 1024 * 1024)
}

fn durability_with_wal_segment_bytes(
    media: &MediaOwnedPhysicalRuntime,
    segment_bytes: u64,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(media, segment_bytes, 64 * 1024 * 1024)
}

fn durability_with_limits(
    media: &MediaOwnedPhysicalRuntime,
    segment_bytes: u64,
    retained_wal_tail_bytes: u64,
) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media.physical_durability_admission_basis().unwrap();
    let wal = PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(segment_bytes).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).unwrap()),
    );
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(wal)
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(retained_wal_tail_bytes).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy,
        _ => panic!("blob journey durability must admit"),
    }
}

trait BootstrapDenialReason {
    fn reason(&self) -> RecordBootstrapDenial;
}

impl BootstrapDenialReason for RecordStoreInitializationDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

impl BootstrapDenialReason for RecordStoreOpenDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

fn success<Denial>(outcome: RecordServingAdmissionOutcome<Denial>) -> ServingPhysicalRuntime
where
    Denial: BootstrapDenialReason,
{
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => serving,
        TransitionOutcome::Denied(denial) => {
            panic!("blob journey bootstrap denied: {:?}", denial.reason())
        }
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Stale(stale) => {
            panic!("blob journey bootstrap stale: {:?}", stale.reason())
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!(
                "blob journey bootstrap rebind required: {:?}",
                rebind.reason()
            )
        }
        TransitionOutcome::Failed(inspection) => {
            panic!(
                "blob journey bootstrap inspection required: {:?}",
                inspection.cause()
            )
        }
    }
}
