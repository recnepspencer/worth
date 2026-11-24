#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalSnapshotAdmissionDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
}

pub(crate) enum CurrentBranchSnapshotAdmissionDenial {
    Basis(crate::branch::basis_denial::RelationalBranchBasisDenial),
    Snapshot(RelationalSnapshotAdmissionDenial),
    CurrentVersionUnavailable,
}

impl std::fmt::Debug for CurrentBranchSnapshotAdmissionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basis(cause) => f.debug_tuple("Basis").field(cause).finish(),
            Self::Snapshot(cause) => f.debug_tuple("Snapshot").field(cause).finish(),
            Self::CurrentVersionUnavailable => f.write_str("CurrentVersionUnavailable"),
        }
    }
}

mod runtime;
