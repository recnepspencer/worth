use super::AspectContract;
use crate::aspects::contracts::{
    AbsenceLaw, AspectEquivalenceBasis, AspectShape, OpaqueAspectType, ReferenceAspectType,
};
use crate::aspects::evolution::AspectEvolutionPolicy;
use crate::aspects::identity::{AspectContractRevision, AspectIdentity};
use crate::aspects::keys::AspectKey;
use crate::aspects::masks::AspectMaskContract;
use crate::aspects::structs::StructAspectShape;
use crate::values::ScalarAspectType;

impl AspectContract {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
        shape: AspectShape,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> Self {
        Self {
            key,
            identity,
            revision,
            shape,
            masks,
            absence,
            equivalence,
            evolution,
        }
    }

    pub fn scalar(
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
        scalar: ScalarAspectType,
    ) -> Self {
        Self::new(
            key,
            identity,
            revision,
            AspectShape::Scalar(scalar),
            AspectMaskContract::scalar(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
    }

    pub fn struct_aspect(
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
        shape: StructAspectShape,
    ) -> Self {
        Self::new(
            key,
            identity,
            revision,
            AspectShape::Struct(shape),
            AspectMaskContract::struct_fields(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
    }

    pub fn reference_entity(
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    ) -> Self {
        Self::new(
            key,
            identity,
            revision,
            AspectShape::Reference(ReferenceAspectType::Entity),
            AspectMaskContract::scalar(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::ReferenceIdentity,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
    }

    pub fn opaque_token(
        key: AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    ) -> Self {
        Self::new(
            key,
            identity,
            revision,
            AspectShape::Opaque(OpaqueAspectType::Token),
            AspectMaskContract::opaque_diagnostic_only(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::OpaqueIdentity,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
    }
}
