//! Semantic journal DATA. No runtime identity, port, TypeId or live receipt is encoded.
use crate::domain_computation::application_aftermath::correction_material::{
    QueryCorrectionMaterialDenial as Denial, QueryCorrectionMaterialFrame,
};
use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectValue, CorrectionMaterialPinIdentity, CorrectionMaterialPinOwner,
    DurableCompositePublicationAttemptIdentity, RetentionClass,
};
use worth_relational::facade::{
    content::RelationalContentAttachments,
    history::RelationalCommitReceipt,
    identity::{EntityId, KindId},
    transactions::RecordRef,
};
mod output_fields;
mod prepared;
mod scalar;
mod source_facts;
pub(in crate::domain_computation::primary_graph) use output_fields::{
    RecordedAbsenceKind, RecordedFieldCapture, RecordedOutputFields, RecordedPreImageAbsence,
};
pub(in crate::domain_computation::primary_graph) use prepared::{
    PreparedCorrectionSnapshot, RecordedCorrectionTarget,
};
pub(in crate::domain_computation::primary_graph) use source_facts::{
    RecordedSourceFacts, RecordedSourceFactsUnavailable,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedLocation {
    pub(in crate::domain_computation::primary_graph) aspect: String,
    pub(in crate::domain_computation::primary_graph) fields: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedScope {
    pub(in crate::domain_computation::primary_graph) location: RecordedLocation,
    pub(in crate::domain_computation::primary_graph) value: AspectValue,
    pub(in crate::domain_computation::primary_graph) subject: EntityId,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedDisclosure {
    pub(in crate::domain_computation::primary_graph) query: [u8; 32],
    pub(in crate::domain_computation::primary_graph) parameters: Vec<(String, AspectValue)>,
    pub(in crate::domain_computation::primary_graph) identity: [u8; 32],
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedPreImageField {
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) aspect: String,
    pub(in crate::domain_computation::primary_graph) field: String,
    pub(in crate::domain_computation::primary_graph) value: AspectValue,
    pub(in crate::domain_computation::primary_graph) target: RecordRef,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) encoded_bytes: usize,
    pub(in crate::domain_computation::primary_graph) location: RecordedLocation,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct MaterialCapture {
    pub(in crate::domain_computation::primary_graph) original:
        DurableCompositePublicationAttemptIdentity,
    pub(in crate::domain_computation::primary_graph) program: [u8; 32],
    pub(in crate::domain_computation::primary_graph) mutation: String,
    pub(in crate::domain_computation::primary_graph) key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) intent: String,
    pub(in crate::domain_computation::primary_graph) intent_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) input: Vec<u8>,
    pub(in crate::domain_computation::primary_graph) principal: EntityId,
    pub(in crate::domain_computation::primary_graph) scope_entity: EntityId,
    pub(in crate::domain_computation::primary_graph) scope: Option<RecordedScope>,
    pub(in crate::domain_computation::primary_graph) disclosure: Option<RecordedDisclosure>,
    pub(in crate::domain_computation::primary_graph) preimage: Option<Vec<RecordedPreImageField>>,
    pub(in crate::domain_computation::primary_graph) output_fields: Option<RecordedOutputFields>,
    pub(in crate::domain_computation::primary_graph) content: RelationalContentAttachments,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedOutputRole {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) posture: u8,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedOutputField {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) location: RecordedLocation,
    pub(in crate::domain_computation::primary_graph) value: AspectValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct MaterialReady {
    pub(in crate::domain_computation::primary_graph) original:
        DurableCompositePublicationAttemptIdentity,
    pub(in crate::domain_computation::primary_graph) commit: RelationalCommitReceipt,
    pub(in crate::domain_computation::primary_graph) roles: Vec<RecordedOutputRole>,
    pub(in crate::domain_computation::primary_graph) output_fields: Vec<RecordedOutputField>,
    pub(in crate::domain_computation::primary_graph) content: RelationalContentAttachments,
    pub(in crate::domain_computation::primary_graph) source_facts: RecordedSourceFacts,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "record", deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) enum MaterialRecord {
    RecoveryPin {
        identity: CorrectionMaterialPinIdentity,
        original: DurableCompositePublicationAttemptIdentity,
        class: RetentionClass,
    },
    Capture {
        material: Box<MaterialCapture>,
    },
    Ready {
        material: Box<MaterialReady>,
    },
    Pin {
        identity: CorrectionMaterialPinIdentity,
        original: DurableCompositePublicationAttemptIdentity,
        owner: CorrectionMaterialPinOwner,
        prepared: Box<PreparedCorrectionSnapshot>,
        class: RetentionClass,
    },
    Transfer {
        identity: CorrectionMaterialPinIdentity,
        before: CorrectionMaterialPinOwner,
        after: CorrectionMaterialPinOwner,
        class: RetentionClass,
    },
    Release {
        identity: CorrectionMaterialPinIdentity,
        owner: CorrectionMaterialPinOwner,
        class: RetentionClass,
    },
    DischargePreparation {
        identity: CorrectionMaterialPinIdentity,
        key: [u8; 32],
        class: RetentionClass,
    },
    Abandon {
        original: DurableCompositePublicationAttemptIdentity,
    },
    Expire {
        original: DurableCompositePublicationAttemptIdentity,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialEnvelope {
    version: u16,
    body: MaterialRecord,
}
#[derive(Serialize)]
struct EncodedMaterialEnvelope<'a> {
    version: u16,
    body: &'a MaterialRecord,
}
const MATERIAL_RECORD_VERSION: u16 = 3;
impl MaterialRecord {
    pub(in crate::domain_computation::primary_graph) fn encode(&self) -> Result<Vec<u8>, Denial> {
        scalar::validate(self)?;
        serde_json::to_vec(&EncodedMaterialEnvelope {
            version: MATERIAL_RECORD_VERSION,
            body: self,
        })
        .map_err(|_| Denial::MalformedFrame)
    }
    pub(in crate::domain_computation::primary_graph) fn decode(
        frame: &QueryCorrectionMaterialFrame,
    ) -> Result<Self, Denial> {
        let envelope: MaterialEnvelope =
            serde_json::from_slice(frame.body()).map_err(|_| Denial::MalformedFrame)?;
        if envelope.version != MATERIAL_RECORD_VERSION {
            return Err(Denial::UnsupportedVersion(u32::from(envelope.version)));
        }
        scalar::validate(&envelope.body)?;
        if envelope.body.class() != frame.body_retention_class() {
            return Err(Denial::RetentionClassMismatch);
        }
        Ok(envelope.body)
    }
    pub(in crate::domain_computation::primary_graph) fn class(&self) -> RetentionClass {
        match self {
            Self::Capture { .. } => RetentionClass::MandatoryObligation,
            Self::Ready { .. } => RetentionClass::OptionalRetainedMaterial,
            Self::Pin { class, .. } | Self::RecoveryPin { class, .. } => *class,
            Self::Transfer { .. }
            | Self::Release { .. }
            | Self::DischargePreparation { .. }
            | Self::Abandon { .. }
            | Self::Expire { .. } => RetentionClass::Authoritative,
        }
    }
}
