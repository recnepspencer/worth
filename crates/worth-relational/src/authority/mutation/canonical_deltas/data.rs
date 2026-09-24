use smallvec::SmallVec;

use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::schema::data::AspectContractPlanRevision;
use crate::transactions::data::RecordRef;
use crate::transactions::data::{
    AspectDeltaFailureFields, AspectDeltaPatchConstructionDenial, AspectDeltaPatchValueDenial,
    AspectDeltaRecordClass, CommitConflict, ConflictClass,
};
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectFieldLocator, AspectKey, AspectValueLocator,
    ContractValidatedAspectValue, FieldLevelAspectPatch,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalRecordAspectDelta {
    pub(crate) target: RecordRef,
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) plan_revision: AspectContractPlanRevision,
    pub(crate) structural_change: RecordStructuralChange,
    pub(crate) changed_aspects: Vec<AspectKey>,
    pub(crate) evaluated_bindings: SmallVec<[EvaluatedAspectBinding; 4]>,
    pub(crate) contains_opaque_aspect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvaluatedAspectBinding {
    pub(crate) aspect_key: AspectKey,
    pub(crate) contract: worth_foundational::AspectContract,
    pub(crate) binding: AspectBinding,
    pub(crate) changed: bool,
    pub(crate) aspect_shape: worth_foundational::AspectShape,
    pub(crate) evidence: CanonicalAspectDeltaEvidence,
    /// Exact old/new field transitions when patch evidence replaces value evidence.
    /// `None` retains the normal evidence projection; `Some([])` proves no field changed.
    pub(crate) field_revision_changes: Option<
        Vec<(
            worth_foundational::facade::FieldKey,
            crate::storage::data::RelationalFieldPresence,
        )>,
    >,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalAspectDeltaEvidence {
    ScalarAspectValueTransition {
        locator: AspectValueLocator,
        old_present: bool,
        new_present: bool,
        old_value: Option<worth_foundational::facade::AspectValue>,
        new_value: Option<worth_foundational::facade::AspectValue>,
    },
    StructAspectValueTransition {
        locator: AspectValueLocator,
        old_present: bool,
        new_present: bool,
        old_value: Option<worth_foundational::facade::StructAspectValue>,
        new_value: Option<worth_foundational::facade::StructAspectValue>,
    },
    EndpointIdentity {
        locator: AspectValueLocator,
        old: Option<EntityId>,
        new: Option<EntityId>,
    },
    Lifecycle {
        locator: AspectValueLocator,
        transition: LifecycleTransitionClass,
    },
    Structural {
        locator: AspectValueLocator,
        change: RecordStructuralChange,
    },
    AuthoritativePatch {
        locator: AspectValueLocator,
        operation: AuthoritativePatchDeltaOperation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthoritativePatchDeltaOperation {
    WholeAspectSet { value: ContractValidatedAspectValue },
    WholeAspectClear { contract: AspectContract },
    FieldLevelPatch { patch: FieldLevelAspectPatch },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecycleTransitionClass {
    NoTransition,
    Create,
    Delete,
    RetainForAudit,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingEvaluationContext<'a> {
    Entity {
        structural_change: RecordStructuralChange,
        old_authoritative_state:
            Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
        new_authoritative_state:
            Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
    },
    Relation {
        structural_change: RecordStructuralChange,
        old_authoritative_state:
            Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
        new_authoritative_state:
            Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState>,
        old_source: Option<EntityId>,
        new_source: Option<EntityId>,
        old_target: Option<EntityId>,
        new_target: Option<EntityId>,
    },
}

impl<'a> BindingEvaluationContext<'a> {
    pub(crate) fn structural_change(self) -> RecordStructuralChange {
        match self {
            Self::Entity {
                structural_change, ..
            }
            | Self::Relation {
                structural_change, ..
            } => structural_change,
        }
    }

    pub(crate) fn old_authoritative_state(
        self,
    ) -> Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState> {
        match self {
            Self::Entity {
                old_authoritative_state,
                ..
            } => old_authoritative_state,
            Self::Relation {
                old_authoritative_state,
                ..
            } => old_authoritative_state,
        }
    }

    pub(crate) fn new_authoritative_state(
        self,
    ) -> Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState> {
        match self {
            Self::Entity {
                new_authoritative_state,
                ..
            } => new_authoritative_state,
            Self::Relation {
                new_authoritative_state,
                ..
            } => new_authoritative_state,
        }
    }

    pub(crate) fn relation_endpoints(
        self,
    ) -> Option<(
        Option<EntityId>,
        Option<EntityId>,
        Option<EntityId>,
        Option<EntityId>,
    )> {
        match self {
            Self::Entity { .. } => None,
            Self::Relation {
                old_source,
                new_source,
                old_target,
                new_target,
                ..
            } => Some((old_source, new_source, old_target, new_target)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonicalDeltaError {
    MissingEntityAspectPlan {
        kind_id: crate::identity::data::KindId,
    },
    MissingRelationAspectPlan {
        kind_id: crate::identity::data::KindId,
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

impl CanonicalDeltaError {
    pub(crate) fn to_commit_conflict(&self) -> CommitConflict {
        CommitConflict::new(ConflictClass::AspectDeltaFailure {
            detail: self.detail(),
            fields: self.failure_fields(),
        })
    }

    fn detail(&self) -> String {
        match self {
            Self::MissingEntityAspectPlan { kind_id } => format!(
                "missing lowered entity aspect plan for kind {} during canonical delta evaluation",
                kind_id.0
            ),
            Self::MissingRelationAspectPlan { kind_id } => format!(
                "missing lowered relation aspect plan for kind {} during canonical delta evaluation",
                kind_id.0
            ),
            Self::InvalidLoweredBindingForRecordClass { detail, .. } => detail.clone(),
            Self::AspectValueMaterialization { detail, .. } => detail.clone(),
            Self::EntityFieldBindingRequiresAuthoritativePatchEvidence { target } => format!(
                "entity aspect field target {} requires authoritative patch evidence during canonical delta evaluation",
                hex_bytes(&crate::aspect_wire::encode_aspect_field_locator(target))
            ),
            Self::FoundationalPatchValueValidation {
                target,
                aspect_key,
                denial,
            } => format!(
                "failed to validate foundational patch value for aspect {:?} on {:?}: {:?}",
                aspect_key, target, denial
            ),
            Self::FoundationalPatchConstruction { target, denial } => format!(
                "failed to materialize foundational patch fragment for {:?}: {:?}",
                target, denial
            ),
        }
    }

    fn failure_fields(&self) -> AspectDeltaFailureFields {
        match self {
            Self::MissingEntityAspectPlan { kind_id } => {
                AspectDeltaFailureFields::MissingAspectPlan {
                    kind_id: *kind_id,
                    record_class: AspectDeltaRecordClass::Entity,
                    code: crate::diagnostics::data::DiagnosticCode::AspectDeltaFailure,
                }
            }
            Self::MissingRelationAspectPlan { kind_id } => {
                AspectDeltaFailureFields::MissingAspectPlan {
                    kind_id: *kind_id,
                    record_class: AspectDeltaRecordClass::Relation,
                    code: crate::diagnostics::data::DiagnosticCode::AspectDeltaFailure,
                }
            }
            Self::InvalidLoweredBindingForRecordClass { aspect_key, detail } => {
                AspectDeltaFailureFields::InvalidLoweredBindingForRecordClass {
                    aspect_key: aspect_key.clone(),
                    detail: detail.clone(),
                }
            }
            Self::AspectValueMaterialization { aspect_key, detail } => {
                AspectDeltaFailureFields::AspectValueMaterialization {
                    aspect_key: aspect_key.clone(),
                    detail: detail.clone(),
                }
            }
            Self::EntityFieldBindingRequiresAuthoritativePatchEvidence { target } => {
                AspectDeltaFailureFields::EntityFieldBindingRequiresAuthoritativePatchEvidence {
                    target: target.clone(),
                }
            }
            Self::FoundationalPatchValueValidation {
                target,
                aspect_key,
                denial,
            } => AspectDeltaFailureFields::FoundationalPatchValueValidation {
                target: target.clone(),
                aspect_key: aspect_key.clone(),
                denial: denial.clone(),
            },
            Self::FoundationalPatchConstruction { target, denial } => {
                AspectDeltaFailureFields::FoundationalPatchConstruction {
                    target: target.clone(),
                    denial: denial.clone(),
                }
            }
        }
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
