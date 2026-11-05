//! Moved whole from `domain_computation::primary_graph::application_query::resource_lifecycle::basis_registry`: the items below and every impl they own.

pub enum WorthQueryApplicationBasisRegistrationDenial {
    Basis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    Snapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
}
