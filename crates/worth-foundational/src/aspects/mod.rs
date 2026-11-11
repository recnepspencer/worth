mod binding;
mod contracts;
mod evolution;
mod front_doors;
mod identity;
mod keys;
mod masks;
mod patches;
mod portable;
mod state;
mod structs;
mod validation;
mod value_posture;

pub use binding::{AspectBinding, AuthoritativeAspectChangeKind, TruthPartitionRole};
pub use contracts::{
    AbsenceLaw, AspectContract, AspectEquivalenceBasis, AspectShape, OpaqueAspectType,
    ReferenceAspectType,
};
pub use evolution::{
    classify_aspect_contract_evolution, AspectEvolutionClassified,
    AspectEvolutionClassifiedContractArtifact, AspectEvolutionClassifiedContracts,
    AspectEvolutionKind, AspectEvolutionPolicy, AspectEvolutionVerdict,
};
pub use front_doors::{
    aspects, AspectContractFrontDoor, AspectContractIdentityStep, AspectContractRevisionStep,
    AspectContractShapeStep, AspectFrontDoorConstructionDenial, AspectMaskContractFrontDoor,
    AspectPatchFrontDoor, AspectValidationFrontDoor, AspectValidationInputStep,
    AspectVocabularyFrontDoor, AspectsFrontDoor, AuthoritativeStateFrontDoor,
    DiagnosticMaskFrontDoor, FieldLevelPatchBuilder, MutationMaskFrontDoor,
    ProjectionMaskFrontDoor, StructFieldBuilder, StructFieldsFrontDoor, StructValueBuilder,
    WholeAspectPatchBuilder,
};
pub use identity::{AspectContractRevision, AspectIdentity};
pub use keys::AspectKey;
pub use masks::{
    AspectMask, AspectMaskContract, DiagnosticMask, MaskAdmissibilityDenial, MutationMask,
    ProjectionMask,
};
pub use patches::{
    AspectPatchStorageBounds, AuthoritativePatchApplicationDenial,
    AuthoritativePatchConstructionDenial, AuthoritativeRecordAspectPatch, FieldLevelAspectPatch,
};
pub use portable::{
    export_portable_record_aspect_patch, export_portable_record_aspect_state,
    readmit_portable_record_aspect_patch, readmit_portable_record_aspect_state,
    PortableAspectContract, PortableAspectContractBasis, PortableAspectContractDenial,
    PortableAspectContractLookup, PortableAspectExportDenial, PortableAspectFieldSet,
    PortableAspectPatchOperation, PortableAspectReadmissionDenial, PortablePatchReadmissionPurpose,
    PortableRecordAspectPatch, PortableRecordAspectState, PortableRecordAspectStateEntry,
};
pub use state::{
    admit_authoritative_record_aspect_state, AuthoritativeRecordAspectState,
    AuthoritativeRecordAspectStateAdmitted, AuthoritativeRecordAspectStateArtifact,
    AuthoritativeStateAdmissionDenial, CanonicalAspectStateMap,
};
pub use structs::{
    CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement, StructAspectShape,
    StructAspectValue, StructAspectValueConstructionDenial, StructAspectValueSortedFieldsDenied,
    StructAspectValueStorageBounds,
};
pub use validation::{
    validate_aspect_value, ContractValidatedAspectArtifact, ContractValidatedAspectValue,
    ContractValidatedAspectValueView, ContractValidationDenial, ContractValidationInput,
};
pub use value_posture::AspectValuePosture;

use crate::facade::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "aspect_state_and_patches",
        "aspect contracts, state-map vocabulary, mask law, and patch vocabulary",
        "domain-owned truth mutation or persistence engines",
    )
}
