//! Historical correction meaning is DATA; it is never a reconstructed live receipt.
use super::*;
use crate::domain_computation::application_aftermath::{
    OriginalCorrectionDisclosure, WorthQueryCommittedAftermathCausality, WorthQueryRetainedPreImage,
};
use crate::domain_computation::primary_graph::{
    application_checkpoint::facts::FactPayload, retained_correction::RetainedCorrectionMaterial,
    OriginalProgramMeaning, WorthQueryApplicationOutputCorrespondence,
    WorthQueryObservedScopeSelector,
};
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::WorthQueryInstalledAftermathContract;
use worth_relational::facade::history::RelationalCommitReceipt;
mod presence;
mod readmission;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum CorrectionOccurrence {
    Memory(Arc<WorthQueryPrimaryGraphCommittedApplication>),
    Home(Arc<HomeCorrectionOccurrence>),
}
pub(in crate::domain_computation::primary_graph) struct HomeCorrectionOccurrence {
    pub(super) material: RetainedCorrectionMaterial,
    pub(super) binding: std::any::TypeId,
    pub(super) program: Arc<OriginalProgramMeaning>,
    pub(super) aftermath: WorthQueryInstalledAftermathContract,
    pub(super) disclosure: OriginalCorrectionDisclosure,
    pub(super) scope: Arc<WorthQueryObservedScopeSelector>,
    pub(super) correspondence: WorthQueryApplicationOutputCorrespondence,
    pub(super) causality: Option<OriginalCorrectionLineage>,
}

/// Source-backed descriptive lineage, with identities read from this receiver's history.
pub(super) struct OriginalCorrectionLineage {
    pub(super) role:
        crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole,
    pub(super) lineage:
        crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding,
}
pub(super) enum CorrectionLineageObservation<'a> {
    Memory(&'a WorthQueryCommittedAftermathCausality),
    Home(&'a OriginalCorrectionLineage),
}
impl<'a> CorrectionLineageObservation<'a> {
    pub(super) fn role(
        &self,
    ) -> crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole {
        match self {
            Self::Memory(value) => value.role(),
            Self::Home(value) => value.role,
        }
    }
    pub(super) fn correction_lineage(
        &self,
    ) -> &'a crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding
    {
        match self {
            Self::Memory(value) => value.correction_lineage(),
            Self::Home(value) => &value.lineage,
        }
    }
}

impl CorrectionOccurrence {
    pub(super) fn memory(&self) -> Option<&WorthQueryPrimaryGraphCommittedApplication> {
        match self {
            Self::Memory(value) => Some(value),
            Self::Home(_) => None,
        }
    }
    pub(super) fn commit_reference(&self) -> &RelationalCommitReceipt {
        match self {
            Self::Memory(value) => value.commit_reference(),
            Self::Home(value) => &value.material.ready.commit,
        }
    }
    pub(super) fn original_binding(&self) -> Option<std::any::TypeId> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .original_mutation_binding(),
            Self::Home(value) => Some(value.binding),
        }
    }
    pub(super) fn original_program(&self) -> Option<&OriginalProgramMeaning> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .original_program(),
            Self::Home(value) => Some(&value.program),
        }
    }
    pub(super) fn disclosure(&self) -> Option<&OriginalCorrectionDisclosure> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .retained_governed_input_carrier()
                .and_then(|input| input.correction()),
            Self::Home(value) => Some(&value.disclosure),
        }
    }
    pub(super) fn scope(&self) -> Option<&WorthQueryObservedScopeSelector> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .retained_governed_input_carrier()
                .and_then(|input| input.original_scope())
                .map(AsRef::as_ref),
            Self::Home(value) => Some(&value.scope),
        }
    }
    pub(super) fn aftermath(&self) -> Option<&WorthQueryInstalledAftermathContract> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .installed_aftermath(),
            Self::Home(value) => Some(&value.aftermath),
        }
    }
    pub(super) fn original_intent(&self) -> [u8; 32] {
        match self {
            Self::Memory(value) => *value
                .commit_evidence()
                .original_authority()
                .idempotency_binding()
                .intent_identity(),
            Self::Home(value) => value.material.capture.intent_identity,
        }
    }
    pub(super) fn input(&self) -> Option<(&[u8], [u8; 32])> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .original_authority()
                .retained_governed_input_carrier()
                .and_then(|input| input.material_input())
                .map(|(input, _, identity)| (input.recorded_bytes(), identity)),
            Self::Home(value) => Some((
                &value.material.capture.input,
                value.material.capture.input_identity,
            )),
        }
    }
    pub(super) fn correspondence(&self) -> &WorthQueryApplicationOutputCorrespondence {
        match self {
            Self::Memory(value) => value.commit_evidence().output_correspondence(),
            Self::Home(value) => &value.correspondence,
        }
    }
    pub(super) fn field<'a>(
        &'a self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entity: worth_relational::facade::identity::EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<&'a AspectValue>, worth_execution::MapKernelStop> {
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .committed_changes()
                .committed_field_ref(request, entity, locator),
            Self::Home(value) => {
                request.check_live()?;
                Ok(value
                    .material
                    .ready
                    .output_fields
                    .iter()
                    .find(|field| {
                        field.entity == entity
                            && field.location.aspect == locator.aspect().aspect_key().as_str()
                            && field.location.fields.iter().map(String::as_str).eq(locator
                                .field_path()
                                .fields()
                                .iter()
                                .map(|field| field.as_str()))
                    })
                    .map(|field| &field.value))
            }
        }
    }
    pub(super) fn causality(&self) -> Option<CorrectionLineageObservation<'_>> {
        match self {
            Self::Memory(value) => value
                .aftermath_causality()
                .map(CorrectionLineageObservation::Memory),
            Self::Home(value) => value
                .causality
                .as_ref()
                .map(CorrectionLineageObservation::Home),
        }
    }
    pub(super) fn memory_preimage(&self) -> Option<&WorthQueryRetainedPreImage> {
        self.memory()
            .and_then(|value| value.commit_evidence().retained_preimage())
    }
    pub(super) fn home_material(&self) -> Option<&RetainedCorrectionMaterial> {
        match self {
            Self::Memory(_) => None,
            Self::Home(value) => Some(&value.material),
        }
    }
    pub(super) fn has_preimage(&self) -> bool {
        match self {
            Self::Memory(_) => self.memory_preimage().is_some(),
            Self::Home(value) => value.material.capture.preimage.is_some(),
        }
    }
    pub(super) fn preimage_value(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectValue> {
        match self {
            Self::Memory(_) => self
                .memory_preimage()?
                .fields()
                .iter()
                .find(|value| {
                    value.locus().entity() == entity
                        && value.locus().aspect() == aspect
                        && value.locus().field() == field
                })
                .map(|value| value.value()),
            Self::Home(value) => value
                .material
                .capture
                .preimage
                .as_ref()?
                .iter()
                .find(|value| {
                    value.entity == entity && value.aspect == aspect && value.field == field
                })
                .map(|value| &value.value),
        }
    }
    pub(super) fn preimage_key(
        &self,
        entity: worth_relational::facade::identity::EntityId,
        name: &str,
        locator: &AspectFieldLocator,
    ) -> Option<&AspectValue> {
        match self {
            Self::Memory(_) => self
                .memory_preimage()?
                .fields()
                .iter()
                .find(|value| {
                    value.target_record()
                        == &worth_relational::facade::transactions::RecordRef::Entity(entity)
                        && value.locator() == locator
                        && value.locus().entity() == name
                })
                .map(|value| value.value()),
            Self::Home(value) => value
                .material
                .capture
                .preimage
                .as_ref()?
                .iter()
                .find(|value| {
                    value.target
                        == worth_relational::facade::transactions::RecordRef::Entity(entity)
                        && value.entity == name
                        && value.location.aspect == locator.aspect().aspect_key().as_str()
                        && value.location.fields.iter().map(String::as_str).eq(locator
                            .field_path()
                            .fields()
                            .iter()
                            .map(|field| field.as_str()))
                })
                .map(|value| &value.value),
        }
    }
    pub(super) fn preimage_content(
        &self,
    ) -> Option<&worth_relational::facade::content::RelationalContentAttachments> {
        match self {
            Self::Memory(_) => self.memory_preimage().map(|value| value.content()),
            Self::Home(value) => value
                .material
                .capture
                .preimage
                .as_ref()
                .map(|_| &value.material.capture.content),
        }
    }
    pub(super) fn facts(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<FactPayload, ()> {
        use crate::domain_computation::primary_graph::application_checkpoint::facts;
        use crate::domain_computation::primary_graph::retained_correction::record::RecordedSourceFacts;
        match self {
            Self::Memory(value) => value
                .commit_evidence()
                .rebased_source_facts()
                .map_err(|_| ()),
            Self::Home(value) => match &value.material.ready.source_facts {
                RecordedSourceFacts::Exact { version, bytes } => facts::decode_for_wire_version(
                    bytes,
                    *version,
                    None,
                    request.allocation_policy(),
                )
                .map_err(|_| ()),
                RecordedSourceFacts::Empty => {
                    use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
                    facts::NativeFactPayloadBuilder::allocate(
                        0,
                        StorageControl::new(request.allocation_policy(), None),
                    )
                    .map_err(|_| ())?
                    .seal()
                    .map_err(|_| ())
                }
                RecordedSourceFacts::Unavailable(_) => Err(()),
            },
        }
    }
}
