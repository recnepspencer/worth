use crate::domain_installation::{WorthQueryAdmittedLocality, WorthQueryImpactClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMaintenanceStrategy {
    Suppression,
    LocalProjectionPatch,
    MembershipSplice,
    StableReorderOrRegroup,
    WindowRefill,
    BoundedReexecution,
    ExplicitRebind,
    Replacement,
    Retirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryMaintenanceScope {
    ExactSourceRecord {
        partition_id: u32,
        local_slot: u64,
        generation: u32,
    },
    SourcePartition(String),
    WholeLogicalGraph,
}

pub(super) fn strategy_for(class: WorthQueryImpactClass) -> Option<WorthQueryMaintenanceStrategy> {
    match class {
        WorthQueryImpactClass::UnaffectedOrSuppressed => {
            Some(WorthQueryMaintenanceStrategy::Suppression)
        }
        WorthQueryImpactClass::ValuePatch => {
            Some(WorthQueryMaintenanceStrategy::LocalProjectionPatch)
        }
        WorthQueryImpactClass::MembershipSplice => {
            Some(WorthQueryMaintenanceStrategy::MembershipSplice)
        }
        WorthQueryImpactClass::ReorderOrRegroup => {
            Some(WorthQueryMaintenanceStrategy::StableReorderOrRegroup)
        }
        WorthQueryImpactClass::WindowShift => Some(WorthQueryMaintenanceStrategy::WindowRefill),
        WorthQueryImpactClass::Reexecute => Some(WorthQueryMaintenanceStrategy::BoundedReexecution),
        WorthQueryImpactClass::ExplicitRebind => {
            Some(WorthQueryMaintenanceStrategy::ExplicitRebind)
        }
        WorthQueryImpactClass::Replacement => Some(WorthQueryMaintenanceStrategy::Replacement),
        WorthQueryImpactClass::Retirement => Some(WorthQueryMaintenanceStrategy::Retirement),
        WorthQueryImpactClass::UnsupportedEscalation => None,
    }
}

pub(super) fn scope_for(locality: WorthQueryAdmittedLocality) -> WorthQueryMaintenanceScope {
    match locality {
        WorthQueryAdmittedLocality::ExactSourceRecord {
            partition_id,
            local_slot,
            generation,
        } => WorthQueryMaintenanceScope::ExactSourceRecord {
            partition_id,
            local_slot,
            generation,
        },
        WorthQueryAdmittedLocality::SourcePartition(partition) => {
            WorthQueryMaintenanceScope::SourcePartition(partition)
        }
        WorthQueryAdmittedLocality::WholeLogicalGraph => {
            WorthQueryMaintenanceScope::WholeLogicalGraph
        }
    }
}
#[cfg(test)]
#[path = "admission_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
