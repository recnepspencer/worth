use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId};
use crate::publication::patch::data::{
    PatchStreamPosition, PublishedAuthoritativePatch, RecordStructuralChange,
};
use crate::schema::data::AspectContractPlanRevision;

use crate::transactions::data::primitives::RecordRef;
use worth_foundational::facade::{AspectKey, AspectValue, AspectValueLocator, StructAspectValue};

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct AspectEvaluationTrace {
    pub target: RecordRef,
    pub kind_id: KindId,
    pub plan_revision: AspectContractPlanRevision,
    pub structural_change: RecordStructuralChange,
    pub changed_aspects: Vec<AspectKey>,
    pub contains_opaque_aspect: bool,
    pub binding_rows: Vec<AspectEvaluationTraceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct AspectEvaluationTraceRow {
    pub aspect_key: AspectKey,
    pub changed: bool,
    pub aspect_shape: worth_foundational::AspectShape,
    pub evidence: AspectTraceEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectTraceEvidence {
    ScalarAspectPresenceOrValue {
        old_present: bool,
        new_present: bool,
        old_value: Option<AspectValue>,
        new_value: Option<AspectValue>,
    },
    StructAspectPresenceOrValue {
        old_present: bool,
        new_present: bool,
        old_value: Option<StructAspectValue>,
        new_value: Option<StructAspectValue>,
    },
    EndpointIdentity {
        old: Option<EntityId>,
        new: Option<EntityId>,
    },
    Lifecycle {
        transition: AspectLifecycleTransitionClass,
    },
    Structural {
        change: RecordStructuralChange,
    },
    AuthoritativePatch {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value_locator")]
        locator: AspectValueLocator,
        patch: PublishedAuthoritativePatch,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectLifecycleTransitionClass {
    NoTransition,
    Create,
    Update,
    Delete,
    RetainForAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct AspectEmissionTrace {
    pub target: RecordRef,
    pub patch_position: PatchStreamPosition,
    pub patch_record_index: u64,
    pub structural_change: RecordStructuralChange,
    pub changed_aspects: Vec<AspectKey>,
    pub contains_opaque_aspect: bool,
}

#[cfg(test)]
mod tests {
    use crate::transactions::data::aspect_traces::{
        AspectLifecycleTransitionClass, AspectTraceEvidence,
    };

    #[test]
    fn lifecycle_no_transition_is_typed_trace_evidence() {
        let evidence = AspectTraceEvidence::Lifecycle {
            transition: AspectLifecycleTransitionClass::NoTransition,
        };

        let AspectTraceEvidence::Lifecycle { transition } = evidence else {
            panic!("expected lifecycle trace evidence");
        };
        assert_eq!(transition, AspectLifecycleTransitionClass::NoTransition);
    }
}
