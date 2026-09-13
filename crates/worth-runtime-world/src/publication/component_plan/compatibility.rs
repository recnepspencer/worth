use super::{
    LoweredOwnerComponentPlan, RelationalComponentPlan, RelationalComponentPlanPosture,
    SignalComponentPlan, SignalComponentPlanPosture,
};

/// The complete publication posture matrix, checked against the plan's own
/// admitted head. This is internal consistency, not staleness: it asks whether
/// the two per-owner postures agree with the component intent they were
/// lowered from, and whether each leg still pins the component basis its head
/// carries. Staleness is a separate question about the live reference cell and
/// is answered by `current_product_head_is`.
///
/// Compatibility is decided per component against the admitted component
/// intent, so a Retain/Retain plan is consistent only with an intent that
/// changes neither component. `CompositeComponentIntent` has no such value,
/// which is what makes a both-retained publication unreservable rather than
/// merely unreachable through the caller-facing constructors.
pub(super) fn plan_is_internally_consistent(plan: &LoweredOwnerComponentPlan) -> bool {
    let basis = plan.expected().expected().basis();
    if plan.relational().expected() != basis.relational_basis()
        || plan.signal().expected().admission_identity()
            != basis.signal_basis().admission_identity()
    {
        return false;
    }
    let intent = plan.expected().intent();
    relational_plan_is_compatible(plan.relational(), intent.changes_relational())
        && signal_plan_is_compatible(plan.signal(), intent.changes_signal())
}

fn relational_plan_is_compatible(plan: &RelationalComponentPlan, changes: bool) -> bool {
    match plan.posture() {
        RelationalComponentPlanPosture::RetainExact => {
            !changes && plan.prepared_candidate().is_none()
        }
        RelationalComponentPlanPosture::PublishPrepared => {
            changes
                && plan.prepared_candidate().is_some_and(|candidate| {
                    candidate.branch() == plan.expected().identity().branch_id()
                })
        }
    }
}

fn signal_plan_is_compatible(plan: &SignalComponentPlan, changes: bool) -> bool {
    match plan.posture() {
        SignalComponentPlanPosture::RetainExact => !changes,
        SignalComponentPlanPosture::AdvanceExact => changes,
    }
}
