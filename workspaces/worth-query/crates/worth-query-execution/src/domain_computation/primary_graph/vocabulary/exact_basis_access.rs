//! `exact_basis_access`'s vocabulary, moved whole from `primary_graph::exact_basis_access`: its root file's items here, one child per other source file.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExactBasisSnapshotDenial {
    BranchIdentityUnavailable,
    BranchObservationUnavailable,
    BranchMaterializationSuspended,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
}

impl From<RelationalSnapshotAdmissionDenial> for WorthQueryExactBasisSnapshotDenial {
    fn from(denial: RelationalSnapshotAdmissionDenial) -> Self {
        match denial {
            RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            } => Self::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
            RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                Self::SnapshotIdentityExhausted
            }
        }
    }
}
use worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial;
