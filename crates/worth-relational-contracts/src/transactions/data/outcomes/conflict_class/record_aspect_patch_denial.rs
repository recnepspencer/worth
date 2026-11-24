use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AuthoritativePatchApplicationDenial,
    PortableAspectReadmissionDenial,
};

use crate::identity::data::{EntityId, KindId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordAspectPatchTarget {
    EntityCreation {
        kind_id: KindId,
    },
    RelationCreation {
        kind_id: KindId,
    },
    Entity {
        entity_id: EntityId,
        kind_id: KindId,
    },
    Relation {
        relation_id: RelationId,
        kind_id: KindId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordAspectPatchDenial {
    MissingAspectPlan {
        kind_id: KindId,
    },
    EmptyFieldAuthoringPatch,
    FieldAuthoringDenied {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        target: AspectFieldLocator,
        reason: super::AspectFieldTargetRejectionReason,
    },
    StructValueConstructionDenied {
        aspect_key: AspectKey,
    },
    ReadmissionDenied(PortableAspectReadmissionDenial),
    ApplicationDenied(AuthoritativePatchApplicationDenial),
}

pub(super) fn denial_detail(
    target: RecordAspectPatchTarget,
    denial: &RecordAspectPatchDenial,
) -> String {
    let target = match target {
        RecordAspectPatchTarget::EntityCreation { kind_id } => {
            format!("entity creation for kind {}", kind_id.0)
        }
        RecordAspectPatchTarget::RelationCreation { kind_id } => {
            format!("relation creation for kind {}", kind_id.0)
        }
        RecordAspectPatchTarget::Entity { entity_id, .. } => format!("entity {entity_id:?}"),
        RecordAspectPatchTarget::Relation { relation_id, .. } => {
            format!("relation {relation_id:?}")
        }
    };
    match denial {
        RecordAspectPatchDenial::MissingAspectPlan { kind_id } => {
            format!("{target} has no lowered aspect plan for kind {}", kind_id.0)
        }
        RecordAspectPatchDenial::EmptyFieldAuthoringPatch => {
            format!("{target} field authoring patch is empty")
        }
        RecordAspectPatchDenial::FieldAuthoringDenied {
            target: field_target,
            reason,
        } => format!(
            "{target} field authoring target {field_target:?} was denied: {}",
            reason.label()
        ),
        RecordAspectPatchDenial::StructValueConstructionDenied { aspect_key } => format!(
            "{target} could not construct a native struct value for aspect `{}`",
            aspect_key.as_str()
        ),
        RecordAspectPatchDenial::ReadmissionDenied(denial) => {
            format!("{target} aspect patch readmission was denied: {denial:?}")
        }
        RecordAspectPatchDenial::ApplicationDenied(denial) => {
            format!("{target} authoritative aspect patch application was denied: {denial:?}")
        }
    }
}
