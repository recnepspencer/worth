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
pub mod output_fields;
pub mod prepared;
pub mod scalar;
pub mod source_facts;
pub use output_fields::{
    RecordedAbsenceKind, RecordedFieldCapture, RecordedOutputFields, RecordedPreImageAbsence,
};
pub use prepared::{PreparedCorrectionSnapshot, RecordedCorrectionTarget};
pub use source_facts::{RecordedSourceFacts, RecordedSourceFactsUnavailable};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedLocation {
    pub aspect: String,
    pub fields: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedScope {
    pub location: RecordedLocation,
    pub value: AspectValue,
    pub subject: EntityId,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedDisclosure {
    pub query: [u8; 32],
    pub parameters: Vec<(String, AspectValue)>,
    pub identity: [u8; 32],
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPreImageField {
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub value: AspectValue,
    pub target: RecordRef,
    pub kind: KindId,
    pub encoded_bytes: usize,
    pub location: RecordedLocation,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialCapture {
    pub original: DurableCompositePublicationAttemptIdentity,
    pub program: [u8; 32],
    pub mutation: String,
    pub key: [u8; 32],
    pub intent: String,
    pub intent_identity: [u8; 32],
    pub input_identity: [u8; 32],
    pub input: Vec<u8>,
    pub principal: EntityId,
    pub scope_entity: EntityId,
    pub scope: Option<RecordedScope>,
    pub disclosure: Option<RecordedDisclosure>,
    pub preimage: Option<Vec<RecordedPreImageField>>,
    pub output_fields: Option<RecordedOutputFields>,
    pub content: RelationalContentAttachments,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedOutputRole {
    pub role: String,
    pub posture: u8,
    pub entity_name: String,
    pub entity: EntityId,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedOutputField {
    pub role: String,
    pub entity_name: String,
    pub entity: EntityId,
    pub location: RecordedLocation,
    pub value: AspectValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialReady {
    pub original: DurableCompositePublicationAttemptIdentity,
    pub commit: RelationalCommitReceipt,
    pub roles: Vec<RecordedOutputRole>,
    pub output_fields: Vec<RecordedOutputField>,
    pub content: RelationalContentAttachments,
    pub source_facts: RecordedSourceFacts,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "record", deny_unknown_fields)]
pub enum MaterialRecord {
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
    AbandonPublicationPin {
        identity: CorrectionMaterialPinIdentity,
        original: DurableCompositePublicationAttemptIdentity,
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
    pub fn encode(&self) -> Result<Vec<u8>, Denial> {
        scalar::validate(self)?;
        serde_json::to_vec(&EncodedMaterialEnvelope {
            version: MATERIAL_RECORD_VERSION,
            body: self,
        })
        .map_err(|_| Denial::MalformedFrame)
    }
    pub fn decode(frame: &QueryCorrectionMaterialFrame) -> Result<Self, Denial> {
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
    pub fn class(&self) -> RetentionClass {
        match self {
            Self::Capture { .. } => RetentionClass::MandatoryObligation,
            Self::Ready { .. } => RetentionClass::OptionalRetainedMaterial,
            Self::Pin { class, .. } | Self::RecoveryPin { class, .. } => *class,
            Self::Transfer { .. }
            | Self::Release { .. }
            | Self::DischargePreparation { .. }
            | Self::AbandonPublicationPin { .. }
            | Self::Abandon { .. }
            | Self::Expire { .. } => RetentionClass::Authoritative,
        }
    }
}
