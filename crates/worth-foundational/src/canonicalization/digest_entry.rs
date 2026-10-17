use crate::aspects::AspectKey;
use crate::aspects::{
    AbsenceLaw, AspectContractRevision, AspectEquivalenceBasis, AspectEvolutionPolicy,
    AspectIdentity, FieldKey, FieldRequirement, OpaqueAspectType, ReferenceAspectType,
};
use crate::values::{AspectValue, ScalarAspectType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestAspectShapeKind {
    Scalar,
    Struct,
    Opaque,
    Reference,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestMaskMode {
    Projection,
    Mutation,
    Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestPreparationEntry {
    ContractHeader {
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    },
    ContractShape {
        key: AspectKey,
        shape: CanonicalDigestAspectShapeKind,
    },
    ContractScalarShape {
        key: AspectKey,
        scalar: ScalarAspectType,
    },
    ContractOpaqueShape {
        key: AspectKey,
        opaque: OpaqueAspectType,
    },
    ContractReferenceShape {
        key: AspectKey,
        reference: ReferenceAspectType,
    },
    ContractStructField {
        key: AspectKey,
        field: FieldKey,
        value_type: ScalarAspectType,
        requirement: FieldRequirement,
        absence: AbsenceLaw,
        evolution: AspectEvolutionPolicy,
    },
    ContractFieldCodec {
        key: AspectKey,
        field: FieldKey,
        codec: crate::CanonicalDigestId,
    },
    ContractMaskMode {
        key: AspectKey,
        mode: CanonicalDigestMaskMode,
        allowed: bool,
    },
    ContractAbsenceLaw {
        key: AspectKey,
        absence: AbsenceLaw,
    },
    ContractEquivalenceBasis {
        key: AspectKey,
        equivalence: AspectEquivalenceBasis,
    },
    ContractEvolutionPolicy {
        key: AspectKey,
        evolution: AspectEvolutionPolicy,
    },
    MaskWholeAspect {
        key: AspectKey,
        mode: CanonicalDigestMaskMode,
    },
    MaskFieldPath {
        key: AspectKey,
        mode: CanonicalDigestMaskMode,
        path: crate::aspects::CanonicalFieldPath,
    },
    StateAspect {
        key: AspectKey,
        revision: AspectContractRevision,
    },
    StateScalarValue {
        key: AspectKey,
        value: AspectValue,
    },
    StateStructFieldValue {
        key: AspectKey,
        field: FieldKey,
        value: AspectValue,
    },
    PatchWholeAspectClear {
        key: AspectKey,
    },
    PatchWholeAspectSet {
        key: AspectKey,
        revision: AspectContractRevision,
    },
    PatchWholeAspectScalarValue {
        key: AspectKey,
        value: AspectValue,
    },
    PatchWholeAspectStructFieldValue {
        key: AspectKey,
        field: FieldKey,
        value: AspectValue,
    },
    PatchFieldClear {
        key: AspectKey,
        field: FieldKey,
    },
    PatchFieldSet {
        key: AspectKey,
        field: FieldKey,
        value: AspectValue,
    },
}
