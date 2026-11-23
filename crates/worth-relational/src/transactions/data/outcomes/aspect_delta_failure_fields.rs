use serde::{Deserialize, Serialize};

use crate::diagnostics::data::DiagnosticCode;
use crate::transactions::data::primitives::RecordRef;
use worth_foundational::facade::{AspectFieldLocator, AspectKey};
use worth_foundational::facade::{AuthoritativePatchConstructionDenial, ContractValidationDenial};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectDeltaFailureFields {
    MissingAspectPlan {
        kind_id: crate::identity::data::KindId,
        record_class: AspectDeltaRecordClass,
        code: DiagnosticCode,
    },
    InvalidLoweredBindingForRecordClass {
        aspect_key: AspectKey,
        detail: String,
    },
    AspectValueMaterialization {
        aspect_key: AspectKey,
        detail: String,
    },
    EntityFieldBindingRequiresAuthoritativePatchEvidence {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        target: AspectFieldLocator,
    },
    FoundationalPatchValueValidation {
        target: RecordRef,
        aspect_key: AspectKey,
        denial: AspectDeltaPatchValueDenial,
    },
    FoundationalPatchConstruction {
        target: RecordRef,
        denial: AspectDeltaPatchConstructionDenial,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectDeltaRecordClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectDeltaPatchValueDenial {
    MissingChangedScalarValue,
    MissingChangedStructValue,
    ContractValidationDenied(ContractValidationDenial),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectDeltaPatchConstructionDenial {
    FoundationalPatchConstructionDenied(AuthoritativePatchConstructionDenial),
    AuthoritativePatchEvidenceAlreadyCarriesPatch { aspect_key: AspectKey },
}
