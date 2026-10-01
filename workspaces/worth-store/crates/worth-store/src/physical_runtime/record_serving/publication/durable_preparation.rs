mod cancellation;
mod manifest_capacity_transition;
mod outcome;
mod payload;
mod prepared;
mod scope;

pub use cancellation::{PhysicalPreSealCancellationDenial, PhysicalPreSealCancellationOutcome};
pub use manifest_capacity_transition::PhysicalManifestCapacityTransition;
pub use outcome::{
    PhysicalMutationPreparationDeferred, PhysicalMutationPreparationDenial,
    PhysicalMutationPreparationFailure, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationRebindRequired, PhysicalMutationPreparationStale,
    PhysicalMutationPreparationSuccess,
};
pub(in crate::physical_runtime::record_serving) use payload::{
    prepare_canonical_payload, CanonicalPayloadMaterializationObservation,
    CanonicalPayloadPreparationError, CanonicalRecordAppendPayload,
};
pub use prepared::{
    PhysicalMutationAdmissionDisposition, PhysicalMutationResourceShape, PreparedPhysicalMutation,
};
pub(in crate::physical_runtime) use prepared::{
    PlannedPhysicalMutationParts, PreparedDerivedDirectoryBasis, PreparedPhysicalMutationContext,
    PreparedReleaseHeadBasis, PreparedReuseDeclarationBasis,
};
pub(in crate::physical_runtime::record_serving) use scope::record_append_scope_identity;
