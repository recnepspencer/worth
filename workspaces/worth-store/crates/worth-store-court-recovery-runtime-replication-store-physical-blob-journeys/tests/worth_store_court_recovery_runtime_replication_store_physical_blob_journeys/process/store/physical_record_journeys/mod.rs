#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::*;
use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

macro_rules! initialize_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::durability(&media_owner);
        media_owner.initialize_record_store($request)
    }};
}

macro_rules! open_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::durability(&media_owner);
        media_owner.open_record_store($request)
    }};
}



















































use {worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedRecordAccessPolicy, worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::MediaOwnedPhysicalRuntime, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecordReadObservation, worth_store::physical_runtime::RecordReadSession, worth_store::physical_runtime::RecordServingAdmissionOutcome, worth_store::physical_runtime::RecordStoreInitializationDenial, worth_store::physical_runtime::RecordStoreOpenDenial, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
use {worth_store_physical_backend::FilesystemAccessPosture};
#[path = "residency_writeback_fresh_reopen/mod.rs"]
pub(crate) mod residency_writeback_fresh_reopen;
