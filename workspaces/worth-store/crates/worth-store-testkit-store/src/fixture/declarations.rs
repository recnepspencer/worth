pub use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

pub use worth_proof::TransitionOutcome;
pub use worth_store_authority::require_current_store_authority;
pub use worth_store_contracts::ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY;
pub use worth_store_physical_backend::FilesystemAccessPosture;
pub use {
    worth_foundational::aspects, worth_foundational::AspectValue,
    worth_foundational::ContractValidationInput, worth_foundational::InternedString,
    worth_foundational::ScalarAspectType,
};
pub use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::aspect_native::StoreAspectIdentity,
    worth_store::aspect_native::StorePhysicalAuthorityWitness,
    worth_store::aspect_native::StorePhysicalBoundaryWitness,
    worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
pub use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::CheckpointMemoryLimit,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::GroupCommitDelay,
    worth_store::physical_runtime::GroupCommitLimit,
    worth_store::physical_runtime::IdempotencyRetentionGenerations,
    worth_store::physical_runtime::LiveIdempotencyBindingLimit,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    worth_store::physical_runtime::PendingUnresolvedMutationLimit,
    worth_store::physical_runtime::PhysicalCheckpointPolicy,
    worth_store::physical_runtime::PhysicalDurabilityDeclaration,
    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure,
    worth_store::physical_runtime::PhysicalIdempotencyPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalSignalConstructionFailure,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordBootstrapFailure,
    worth_store::physical_runtime::RecordServingAdmissionOutcome,
    worth_store::physical_runtime::RecordStoreInitializationDenial,
    worth_store::physical_runtime::RecordStoreOpenDenial,
    worth_store::physical_runtime::RetainedWalTailLimit,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
pub use {
    worth_store_security::admit_store_security_scope,
    worth_store_security::StoreAuthenticityRequirement,
    worth_store_security::StoreAuthenticityRequirementClass,
    worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope,
    worth_store_security::StoreKeyVersionPosture,
    worth_store_security::StoreSecurityScopeAdmissionExpectation,
    worth_store_security::StoreSecurityScopeAdmissionRequest,
    worth_store_security::StoreTenantScope,
};

pub fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    (format, placement, access)
}

pub fn placement() -> AdmittedRecordPlacementPolicy {
    configuration().1
}

pub fn serving_from_initialization(root: &Path) -> ServingPhysicalRuntime {
    let (_, placement, _) = configuration();
    serving_from_initialization_with_placement(root, placement)
}

pub fn serving_from_initialization_with_placement(
    root: &Path,
    placement: AdmittedRecordPlacementPolicy,
) -> ServingPhysicalRuntime {
    serving_from_initialization_with_placement_and_tail(root, placement, 64 * 1024 * 1024)
}

pub fn serving_from_initialization_with_format_and_placement(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
) -> ServingPhysicalRuntime {
    let media = media(root);
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let durability = durability(&media);
    success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ))
}

pub fn serving_from_initialization_with_placement_and_tail(
    root: &Path,
    placement: AdmittedRecordPlacementPolicy,
    retained_wal_tail_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_limits(&media, 16 * 1024 * 1024, retained_wal_tail_bytes);
    success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ))
}

pub fn serving_from_initialization_with_wal_segment_bytes(
    root: &Path,
    segment_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability_with_wal_segment_bytes(&media, segment_bytes);
    success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ))
}

pub fn serving_from_open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(
        media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        )),
    )
}

pub fn assert_released_open_requires_recovered_custody(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
) {
    let media = media(root);
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let durability = durability(&media);
    let worth_proof::TransitionOutcome::Denied(denial) = media
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        ))
        .into_raw()
    else {
        panic!("selected release-head root must deny Serving without C8 custody")
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
    );
    denial.into_runtime().close();
}

/// A tier-anchored root whose retained WAL refuses its verified tier custody
/// fails ordinary durability reopen before Serving: only C8 may hand it off.
pub fn assert_anchored_open_requires_recovered_custody(root: &Path) {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let worth_proof::TransitionOutcome::Failed(inspection) = media
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        ))
        .into_raw()
    else {
        panic!("an anchored root without admissible clean custody must not serve")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::RecoveredCheckpointCustodyRequired
                )
            )
        ),
        "unexpected reopen cause: {cause:?}"
    );
}

pub fn serving_from_open_with_retained_wal_tail(
    root: &Path,
    retained_wal_tail_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_limits(&media, 16 * 1024 * 1024, retained_wal_tail_bytes);
    success(
        media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        )),
    )
}

pub fn serving_from_open_with_wal_segment_bytes(
    root: &Path,
    segment_bytes: u64,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability_with_wal_segment_bytes(&media, segment_bytes);
    success(
        media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        )),
    )
}

pub fn admitted_blob_scope(identity_key: &str) -> AdmittedBlobScope {
    let witness =
        worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
            worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
                ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
            )
            .unwrap(),
        )
        .unwrap();
    admitted_blob_scope_with_witness(identity_key, witness)
}

pub fn admitted_blob_scope_for_replay_boundary(identity_key: &str) -> AdmittedBlobScope {
    let witness = worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary_instance(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
            ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY,
        )
        .unwrap(),
    )
    .unwrap();
    admitted_blob_scope_with_witness(identity_key, witness)
}

pub fn admitted_blob_scope_with_witness(
    identity_key: &str,
    witness: StorePhysicalBoundaryWitness,
) -> AdmittedBlobScope {
    let key = aspects().vocabulary().key(identity_key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::ScalarAspectType::String);
    let validated = match aspects().validate().against(&contract).value(
        worth_foundational::ContractValidationInput::from(worth_foundational::AspectValue::String(
            worth_foundational::InternedString::from("blob-authority"),
        )),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        outcome => panic!("blob authority value must validate: {outcome:?}"),
    };
    let state = match aspects().authoritative_state().admit([validated]) {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("blob authority state must admit: {outcome:?}"),
    };
    let boundary = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let authority = require_current_store_authority(boundary);
    let key_scope = worth_store_security::StoreKeyScope::BlobChunkEnvelope;
    let tenant_scope = worth_store_security::StoreTenantScope::TenantPhysicalBoundary;
    let authenticity = worth_store_security::StoreAuthenticityRequirement::required(
        worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
    );
    let custody = worth_store_security::StoreCustodyPosture::InternalStoreCustody;
    let expectation = worth_store_security::StoreSecurityScopeAdmissionExpectation::new(
        key_scope,
        tenant_scope,
        authenticity,
        custody,
    );
    let request = worth_store_security::StoreSecurityScopeAdmissionRequest::new(
        &authority,
        key_scope,
        worth_store_security::StoreKeyVersionPosture::Current,
        tenant_scope,
        authenticity,
        custody,
        expectation,
    );
    let admitted = match admit_store_security_scope(request) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("blob custody scope must admit: {outcome:?}"),
    };
    worth_store::physical_runtime::AdmittedBlobScope::from_store_security(admitted).unwrap()
}

pub fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    match runtime
        .try_admit_filesystem_media(
            worth_store::physical_runtime::FilesystemMediaAdmission::production(
                worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("blob journey media must admit"),
    }
}

pub trait BootstrapDenialReason {
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

pub fn success<Denial>(outcome: RecordServingAdmissionOutcome<Denial>) -> ServingPhysicalRuntime
where
    Denial: BootstrapDenialReason,
{
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("blob journey bootstrap denied: {:?}", denial.reason())
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => match deferred {},
        worth_proof::TransitionOutcome::Stale(stale) => {
            panic!("blob journey bootstrap stale: {:?}", stale.reason())
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!(
                "blob journey bootstrap rebind required: {:?}",
                rebind.reason()
            )
        }
        worth_proof::TransitionOutcome::Failed(inspection) => {
            panic!(
                "blob journey bootstrap inspection required: {:?}",
                inspection.cause()
            )
        }
    }
}

#[path = "declarations/durability_policy.rs"]
mod durability_policy;
pub use durability_policy::*;
