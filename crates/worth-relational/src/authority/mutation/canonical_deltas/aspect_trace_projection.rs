use crate::transactions::data::aspect_traces::AspectEvaluationTrace;
use crate::transactions::data::aspect_traces::AspectEvaluationTraceRow;
use crate::transactions::data::aspect_traces::AspectLifecycleTransitionClass;
use crate::transactions::data::aspect_traces::AspectTraceEvidence;

use super::data::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
    LifecycleTransitionClass,
};
use super::published_patch_projection::published_patch_from_authoritative_delta_operation;

impl CanonicalRecordAspectDelta {
    pub(crate) fn evaluation_trace(&self) -> AspectEvaluationTrace {
        AspectEvaluationTrace {
            target: self.target.clone(),
            kind_id: self.kind_id,
            plan_revision: self.plan_revision,
            structural_change: self.structural_change,
            changed_aspects: self.changed_aspects.clone(),
            contains_opaque_aspect: self.contains_opaque_aspect,
            binding_rows: self
                .evaluated_bindings
                .iter()
                .map(EvaluatedAspectBinding::trace_row)
                .collect(),
        }
    }
}

impl EvaluatedAspectBinding {
    fn trace_row(&self) -> AspectEvaluationTraceRow {
        AspectEvaluationTraceRow {
            aspect_key: self.aspect_key.clone(),
            changed: self.changed,
            aspect_shape: self.aspect_shape.clone(),
            evidence: self.evidence.trace_evidence(&self.binding),
        }
    }
}

impl CanonicalAspectDeltaEvidence {
    fn trace_evidence(
        &self,
        binding: &worth_foundational::facade::AspectBinding,
    ) -> AspectTraceEvidence {
        match self {
            Self::ScalarAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
                ..
            } => AspectTraceEvidence::ScalarAspectPresenceOrValue {
                old_present: *old_present,
                new_present: *new_present,
                old_value: old_value.clone(),
                new_value: new_value.clone(),
            },
            Self::StructAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
                ..
            } => AspectTraceEvidence::StructAspectPresenceOrValue {
                old_present: *old_present,
                new_present: *new_present,
                old_value: old_value.clone(),
                new_value: new_value.clone(),
            },
            Self::EndpointIdentity { old, new, .. } => AspectTraceEvidence::EndpointIdentity {
                old: *old,
                new: *new,
            },
            Self::Lifecycle { transition, .. } => AspectTraceEvidence::Lifecycle {
                transition: transition.trace_transition(),
            },
            Self::Structural { change, .. } => AspectTraceEvidence::Structural { change: *change },
            Self::AuthoritativePatch { locator, operation } => {
                AspectTraceEvidence::AuthoritativePatch {
                    locator: locator.clone(),
                    patch: published_patch_from_authoritative_delta_operation(operation, binding),
                }
            }
        }
    }
}

impl LifecycleTransitionClass {
    fn trace_transition(self) -> AspectLifecycleTransitionClass {
        match self {
            Self::NoTransition => AspectLifecycleTransitionClass::NoTransition,
            Self::Create => AspectLifecycleTransitionClass::Create,
            Self::Delete => AspectLifecycleTransitionClass::Delete,
            Self::RetainForAudit => AspectLifecycleTransitionClass::RetainForAudit,
        }
    }
}
