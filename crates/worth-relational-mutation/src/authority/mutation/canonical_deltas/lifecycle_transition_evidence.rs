use worth_relational_contracts::internal_port::publication::patch::data::record_structural_change::RecordStructuralChange;

use super::data::LifecycleTransitionClass;

pub(super) fn lifecycle_transition(
    structural_change: RecordStructuralChange,
) -> LifecycleTransitionClass {
    match structural_change {
        RecordStructuralChange::Created => LifecycleTransitionClass::Create,
        RecordStructuralChange::Updated => LifecycleTransitionClass::NoTransition,
        RecordStructuralChange::Deleted => LifecycleTransitionClass::Delete,
        RecordStructuralChange::RetainedForAudit => LifecycleTransitionClass::RetainForAudit,
        RecordStructuralChange::MaterializationSuspended
        | RecordStructuralChange::Rematerialized => LifecycleTransitionClass::NoTransition,
        _ => unreachable!("unhandled Relational contract variant"),
    }
}
