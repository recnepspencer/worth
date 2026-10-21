use serde::{Deserialize, Serialize};

use crate::aspects::{
    aspects, AbsenceLaw, AspectContract, AspectContractRevision, AspectEquivalenceBasis,
    AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectMaskContract, AspectShape,
    FieldDeclaration, FieldKey, FieldRequirement, StructAspectShape,
};
use crate::values::ScalarAspectType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAspectContract {
    key: AspectKey,
    identity: AspectIdentity,
    revision: AspectContractRevision,
    shape: PortableAspectShape,
    masks: PortableAspectMaskContract,
    absence: AbsenceLaw,
    equivalence: PortableAspectEquivalence,
    evolution: PortableAspectEvolution,
}

impl PortableAspectContract {
    pub fn from_contract(contract: &AspectContract) -> Self {
        Self {
            key: contract.key().clone(),
            identity: contract.identity(),
            revision: contract.revision(),
            shape: PortableAspectShape::from_shape(contract.shape()),
            masks: PortableAspectMaskContract::from_contract(contract.masks()),
            absence: contract.absence(),
            equivalence: contract.equivalence().into(),
            evolution: contract.evolution().into(),
        }
    }

    pub fn key(&self) -> &AspectKey {
        &self.key
    }

    pub fn identity(&self) -> AspectIdentity {
        self.identity
    }

    pub fn revision(&self) -> AspectContractRevision {
        self.revision
    }

    pub fn readmit(&self) -> Result<AspectContract, PortableAspectContractDenial> {
        let masks = self.masks.readmit()?;
        let step = aspects()
            .contract()
            .for_key(self.key.clone())
            .identified_by(self.identity)
            .at_revision(self.revision);
        let contract = match &self.shape {
            PortableAspectShape::Scalar(scalar) => step.scalar_with(
                *scalar,
                masks,
                self.absence,
                self.equivalence.into(),
                self.evolution.into(),
            ),
            PortableAspectShape::Struct(fields) => step.struct_with(
                readmit_struct_shape(fields)?,
                masks,
                self.absence,
                self.equivalence.into(),
                self.evolution.into(),
            ),
            PortableAspectShape::ReferenceEntity => step.reference_with(
                crate::aspects::ReferenceAspectType::Entity,
                masks,
                self.absence,
                self.equivalence.into(),
                self.evolution.into(),
            ),
            PortableAspectShape::OpaqueToken => step
                .opaque_with(
                    crate::aspects::OpaqueAspectType::Token,
                    masks,
                    self.absence,
                    self.equivalence.into(),
                    self.evolution.into(),
                )
                .map_err(|_| PortableAspectContractDenial::IncompatibleConfiguration)?,
            PortableAspectShape::Content => step.content_ref(),
        };
        if Self::from_contract(&contract) != *self {
            return Err(PortableAspectContractDenial::IncompatibleConfiguration);
        }
        Ok(contract)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum PortableAspectShape {
    Scalar(ScalarAspectType),
    Struct(Vec<PortableFieldDeclaration>),
    ReferenceEntity,
    OpaqueToken,
    Content,
}

impl PortableAspectShape {
    fn from_shape(shape: &AspectShape) -> Self {
        match shape {
            AspectShape::Scalar(scalar) => Self::Scalar(*scalar),
            AspectShape::Struct(shape) => Self::Struct(
                shape
                    .fields()
                    .iter()
                    .map(PortableFieldDeclaration::from_declaration)
                    .collect(),
            ),
            AspectShape::Reference(crate::aspects::ReferenceAspectType::Entity) => {
                Self::ReferenceEntity
            }
            AspectShape::Opaque(crate::aspects::OpaqueAspectType::Token) => Self::OpaqueToken,
            AspectShape::Content => Self::Content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PortableFieldDeclaration {
    key: FieldKey,
    value_type: ScalarAspectType,
    requirement: PortableFieldRequirement,
    absence: AbsenceLaw,
    evolution: PortableAspectEvolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    codec: Option<[u8; 32]>,
}

impl PortableFieldDeclaration {
    fn from_declaration(field: &FieldDeclaration) -> Self {
        Self {
            key: field.key().clone(),
            value_type: field.value_type(),
            requirement: field.requirement().into(),
            absence: field.absence(),
            evolution: field.evolution().into(),
            codec: field.codec().map(|codec| *codec.bytes()),
        }
    }
}

fn readmit_struct_shape(
    fields: &[PortableFieldDeclaration],
) -> Result<StructAspectShape, PortableAspectContractDenial> {
    let declarations = fields
        .iter()
        .map(|field| {
            FieldDeclaration::new(
                field.key.clone(),
                field.value_type,
                field.requirement.into(),
                field.absence,
                field.evolution.into(),
            )
            .map(|declaration| match field.codec {
                Some(codec) => declaration.with_codec(crate::CanonicalDigestId::new(codec)),
                None => declaration,
            })
            .ok_or(PortableAspectContractDenial::InvalidFieldDeclaration)
        })
        .collect::<Result<Vec<_>, _>>()?;
    StructAspectShape::new(declarations)
        .ok_or(PortableAspectContractDenial::DuplicateFieldDeclaration)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PortableFieldRequirement {
    Required,
    Optional,
    Defaulted,
}

impl From<FieldRequirement> for PortableFieldRequirement {
    fn from(value: FieldRequirement) -> Self {
        match value {
            FieldRequirement::Required => Self::Required,
            FieldRequirement::Optional => Self::Optional,
            FieldRequirement::Defaulted => Self::Defaulted,
        }
    }
}

impl From<PortableFieldRequirement> for FieldRequirement {
    fn from(value: PortableFieldRequirement) -> Self {
        match value {
            PortableFieldRequirement::Required => Self::Required,
            PortableFieldRequirement::Optional => Self::Optional,
            PortableFieldRequirement::Defaulted => Self::Defaulted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct PortableAspectMaskContract {
    projection: bool,
    mutation: bool,
    diagnostic: bool,
}

impl PortableAspectMaskContract {
    fn from_contract(contract: &AspectMaskContract) -> Self {
        Self {
            projection: contract.projection_allowed(),
            mutation: contract.mutation_allowed(),
            diagnostic: contract.diagnostic_allowed(),
        }
    }

    fn readmit(self) -> Result<AspectMaskContract, PortableAspectContractDenial> {
        match (self.projection, self.mutation, self.diagnostic) {
            (true, true, true) => Ok(AspectMaskContract::scalar()),
            (false, false, true) => Ok(AspectMaskContract::opaque_diagnostic_only()),
            _ => Err(PortableAspectContractDenial::InvalidMaskContract),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PortableAspectEquivalence {
    ExactCanonicalValue,
    DeclaredStructFields,
    OpaqueIdentity,
    ReferenceIdentity,
    ContentIdentity,
}

impl From<AspectEquivalenceBasis> for PortableAspectEquivalence {
    fn from(value: AspectEquivalenceBasis) -> Self {
        match value {
            AspectEquivalenceBasis::ExactCanonicalValue => Self::ExactCanonicalValue,
            AspectEquivalenceBasis::DeclaredStructFields => Self::DeclaredStructFields,
            AspectEquivalenceBasis::OpaqueIdentity => Self::OpaqueIdentity,
            AspectEquivalenceBasis::ReferenceIdentity => Self::ReferenceIdentity,
            AspectEquivalenceBasis::ContentIdentity => Self::ContentIdentity,
        }
    }
}

impl From<PortableAspectEquivalence> for AspectEquivalenceBasis {
    fn from(value: PortableAspectEquivalence) -> Self {
        match value {
            PortableAspectEquivalence::ExactCanonicalValue => Self::ExactCanonicalValue,
            PortableAspectEquivalence::DeclaredStructFields => Self::DeclaredStructFields,
            PortableAspectEquivalence::OpaqueIdentity => Self::OpaqueIdentity,
            PortableAspectEquivalence::ReferenceIdentity => Self::ReferenceIdentity,
            PortableAspectEquivalence::ContentIdentity => Self::ContentIdentity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PortableAspectEvolution {
    Frozen,
    AdditiveFieldsAllowed,
    WideningAllowed,
    ExplicitBreakRequired,
}

impl From<AspectEvolutionPolicy> for PortableAspectEvolution {
    fn from(value: AspectEvolutionPolicy) -> Self {
        match value {
            AspectEvolutionPolicy::Frozen => Self::Frozen,
            AspectEvolutionPolicy::AdditiveFieldsAllowed => Self::AdditiveFieldsAllowed,
            AspectEvolutionPolicy::WideningAllowed => Self::WideningAllowed,
            AspectEvolutionPolicy::ExplicitBreakRequired => Self::ExplicitBreakRequired,
        }
    }
}

impl From<PortableAspectEvolution> for AspectEvolutionPolicy {
    fn from(value: PortableAspectEvolution) -> Self {
        match value {
            PortableAspectEvolution::Frozen => Self::Frozen,
            PortableAspectEvolution::AdditiveFieldsAllowed => Self::AdditiveFieldsAllowed,
            PortableAspectEvolution::WideningAllowed => Self::WideningAllowed,
            PortableAspectEvolution::ExplicitBreakRequired => Self::ExplicitBreakRequired,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortableAspectContractDenial {
    InvalidMaskContract,
    InvalidFieldDeclaration,
    DuplicateFieldDeclaration,
    IncompatibleConfiguration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_contract_survives_serialization_and_exact_readmission() {
        let contract = aspects()
            .contract()
            .for_key(key("note"))
            .identified_by(AspectIdentity(41))
            .at_revision(AspectContractRevision(7))
            .scalar_with(
                ScalarAspectType::String,
                AspectMaskContract::scalar(),
                AbsenceLaw::Optional,
                AspectEquivalenceBasis::ExactCanonicalValue,
                AspectEvolutionPolicy::Frozen,
            );

        let encoded = serde_json::to_vec(&PortableAspectContract::from_contract(&contract))
            .expect("portable contract serialization");
        let candidate: PortableAspectContract =
            serde_json::from_slice(&encoded).expect("portable contract deserialization");

        assert_eq!(candidate.readmit().unwrap(), contract);
    }

    #[test]
    fn struct_contract_preserves_exact_field_declarations() {
        let shape = StructAspectShape::new([
            declaration("title", FieldRequirement::Required, AbsenceLaw::Required),
            declaration("status", FieldRequirement::Optional, AbsenceLaw::Optional),
        ])
        .unwrap();
        let contract = aspects()
            .contract()
            .for_key(key("summary"))
            .identified_by(AspectIdentity(42))
            .at_revision(AspectContractRevision(3))
            .struct_with(
                shape,
                AspectMaskContract::struct_fields(),
                AbsenceLaw::Required,
                AspectEquivalenceBasis::DeclaredStructFields,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            );

        let candidate = PortableAspectContract::from_contract(&contract);

        assert_eq!(candidate.readmit().unwrap(), contract);
    }

    #[test]
    fn tampered_mask_posture_is_denied_before_authority_exists() {
        let contract = AspectContract::scalar(
            key("count"),
            AspectIdentity(43),
            AspectContractRevision(1),
            ScalarAspectType::Int64,
        );
        let mut candidate = PortableAspectContract::from_contract(&contract);
        candidate.masks.mutation = false;

        assert_eq!(
            candidate.readmit(),
            Err(PortableAspectContractDenial::InvalidMaskContract)
        );
    }

    fn declaration(
        name: &str,
        requirement: FieldRequirement,
        absence: AbsenceLaw,
    ) -> FieldDeclaration {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            ScalarAspectType::String,
            requirement,
            absence,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
    }

    fn key(name: &str) -> AspectKey {
        AspectKey::new(name).unwrap()
    }
}
