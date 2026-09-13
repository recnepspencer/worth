use worth_foundational::{
    AspectContract, AspectValue, AuthoritativeRecordAspectStateArtifact,
    ContractValidatedAspectArtifact, ContractValidatedAspectValueView, FieldKey, InternedString,
    StructAspectValue,
};
use worth_store_aspect_native::{
    StoreAspectBoundaryFact, StoreAspectBoundaryLocator, StoreAspectFieldBoundaryLocator,
    StoreAspectIdentity, StoreAspectPatchBoundaryFact, StoreAspectValueBoundaryLocator,
    StorePhysicalBoundaryWitness,
};
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalSegmentId,
};
use worth_store_physical_isolation::{
    CurrentGenerationPhysicalReference, GenerationCountedPhysicalReference,
};

use crate::native_aspect_fixture_authoring::{
    authored_scalar_string_fixture, authored_segment_header_fixture,
    AuthoredNativeStoreAspectFixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectDerivedSegmentReference {
    reference: CurrentGenerationPhysicalReference,
    physical_witness: StorePhysicalBoundaryWitness,
    boundary_fact: StoreAspectBoundaryFact,
    segment_id: PhysicalSegmentId,
    generation: PhysicalGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeAspectPhysicalReferenceDenial {
    ValidatedValueIsNotStruct,
    MissingSegmentField,
    MissingGenerationField,
    SegmentFieldIsNotRawString,
    GenerationFieldIsNotUInt64,
    InvalidSegmentId,
    InvalidGeneration,
    ReferenceIsNotCurrentGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeStoreAspectFixture {
    authored: AuthoredNativeStoreAspectFixture,
}

impl NativeStoreAspectFixture {
    pub fn segment_header(segment: &str, generation: u64) -> Self {
        Self {
            authored: authored_segment_header_fixture(segment, generation),
        }
    }

    pub fn scalar_string(value: &str) -> Self {
        Self {
            authored: authored_scalar_string_fixture(value),
        }
    }

    pub const fn identity(&self) -> &StoreAspectIdentity {
        &self.authored.identity
    }

    pub const fn contract(&self) -> &AspectContract {
        &self.authored.contract
    }

    pub const fn scalar_value(&self) -> Option<&AspectValue> {
        self.authored.scalar_value.as_ref()
    }

    pub const fn struct_value(&self) -> Option<&StructAspectValue> {
        self.authored.struct_value.as_ref()
    }

    pub const fn validated_value(&self) -> &ContractValidatedAspectArtifact {
        &self.authored.validated_value
    }

    pub const fn authoritative_state(&self) -> &AuthoritativeRecordAspectStateArtifact {
        &self.authored.authoritative_state
    }

    pub const fn boundary_fact(&self) -> &StoreAspectBoundaryFact {
        &self.authored.boundary_fact
    }

    pub const fn patch_boundary_fact(&self) -> &StoreAspectPatchBoundaryFact {
        &self.authored.patch_fact
    }

    pub const fn aspect_locator(&self) -> &StoreAspectBoundaryLocator {
        &self.authored.aspect_locator
    }

    pub const fn value_locator(&self) -> &StoreAspectValueBoundaryLocator {
        &self.authored.value_locator
    }

    pub const fn field_locator(&self) -> Option<&StoreAspectFieldBoundaryLocator> {
        self.authored.field_locator.as_ref()
    }

    pub const fn physical_witness(&self) -> StorePhysicalBoundaryWitness {
        self.authored.physical_witness
    }

    pub fn current_generation_segment_reference(
        &self,
    ) -> Option<CurrentGenerationPhysicalReference> {
        self.derive_current_generation_segment_reference()
            .ok()
            .map(AspectDerivedSegmentReference::into_reference)
    }

    pub fn derive_current_generation_segment_reference(
        &self,
    ) -> Result<AspectDerivedSegmentReference, NativeAspectPhysicalReferenceDenial> {
        let struct_value = match self.validated_value().payload().view() {
            ContractValidatedAspectValueView::Struct(value) => value,
            ContractValidatedAspectValueView::Scalar(_) => {
                return Err(NativeAspectPhysicalReferenceDenial::ValidatedValueIsNotStruct);
            }
        };
        let segment = raw_segment_field(struct_value)?;
        let segment_id = PhysicalSegmentId::from_raw(stable_segment_id(segment))
            .map_err(|_| NativeAspectPhysicalReferenceDenial::InvalidSegmentId)?;
        let generation = PhysicalGeneration::from_raw(generation_field(struct_value)?)
            .map_err(|_| NativeAspectPhysicalReferenceDenial::InvalidGeneration)?;
        let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
            .segment_cell(segment_id)
            .with_segment_generation(generation);
        let reference = GenerationCountedPhysicalReference::from_segment_cell(cell)
            .require_current_generation(generation)
            .map_err(|_| NativeAspectPhysicalReferenceDenial::ReferenceIsNotCurrentGeneration)?;

        Ok(AspectDerivedSegmentReference {
            reference,
            physical_witness: self.physical_witness(),
            boundary_fact: self.boundary_fact().clone(),
            segment_id,
            generation,
        })
    }
}

pub const fn require_native_store_aspect_fixture(
    fixture: &NativeStoreAspectFixture,
) -> &StoreAspectBoundaryFact {
    fixture.boundary_fact()
}

impl AspectDerivedSegmentReference {
    pub const fn reference(&self) -> CurrentGenerationPhysicalReference {
        self.reference
    }

    pub const fn physical_witness(&self) -> StorePhysicalBoundaryWitness {
        self.physical_witness
    }

    pub const fn boundary_fact(&self) -> &StoreAspectBoundaryFact {
        &self.boundary_fact
    }

    pub const fn segment_id(&self) -> PhysicalSegmentId {
        self.segment_id
    }

    pub const fn generation(&self) -> PhysicalGeneration {
        self.generation
    }

    pub fn into_reference(self) -> CurrentGenerationPhysicalReference {
        self.reference
    }
}

fn raw_segment_field(
    value: &StructAspectValue,
) -> Result<&str, NativeAspectPhysicalReferenceDenial> {
    match value.get(&field_key("segment")) {
        Some(AspectValue::String(InternedString::Raw(segment))) => Ok(segment.as_str()),
        Some(AspectValue::String(InternedString::Symbol(_))) => {
            Err(NativeAspectPhysicalReferenceDenial::SegmentFieldIsNotRawString)
        }
        Some(_) => Err(NativeAspectPhysicalReferenceDenial::SegmentFieldIsNotRawString),
        None => Err(NativeAspectPhysicalReferenceDenial::MissingSegmentField),
    }
}

fn generation_field(value: &StructAspectValue) -> Result<u64, NativeAspectPhysicalReferenceDenial> {
    match value.get(&field_key("generation")) {
        Some(AspectValue::UInt64(generation)) => Ok(*generation),
        Some(_) => Err(NativeAspectPhysicalReferenceDenial::GenerationFieldIsNotUInt64),
        None => Err(NativeAspectPhysicalReferenceDenial::MissingGenerationField),
    }
}

fn stable_segment_id(segment: &str) -> u64 {
    let mut digest = 0xcbf29ce484222325_u64;
    for byte in segment.as_bytes() {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest.max(1)
}

fn field_key(raw: &str) -> FieldKey {
    FieldKey::new(raw).expect("native segment-header field keys are static and valid")
}
