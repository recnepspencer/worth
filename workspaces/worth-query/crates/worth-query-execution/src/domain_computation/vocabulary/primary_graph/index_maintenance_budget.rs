//! Moved whole from `domain_computation::primary_graph::index_maintenance_budget`: the items below and every impl they own.
use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;

/// Ordinary commit publication pays for changed records, bounded adjacency,
/// and persistent entry paths. It never admits a full-root reconstruction.
pub(in crate::domain_computation) const fn ordinary_index_maintenance_budget(
) -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    }
}
