use super::super::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};

#[cfg(test)]
pub(in crate::domain_computation::primary_graph::application_query) fn map_basis_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::MalformedDescriptor
        | worth_relational::facade::branch::RelationalBranchBasisDenial::UnsupportedDescriptorVersion { .. } => {
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::ForeignRuntime { .. } => {
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::Cancelled => {
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::TimedOut => {
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::StaleReferenceGeneration
        | worth_relational::facade::branch::RelationalBranchBasisDenial::WrongBranchLocalTruthVersion
        | worth_relational::facade::branch::RelationalBranchBasisDenial::EmptyCommittedTargetMismatch
        | worth_relational::facade::branch::RelationalBranchBasisDenial::WrongImmutableTarget
        | worth_relational::facade::branch::RelationalBranchBasisDenial::MixedAxis(_) => {
            WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::UnknownBranch(_)
        | worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(_)
        | worth_relational::facade::branch::RelationalBranchBasisDenial::DeletingBranch(_)
        | worth_relational::facade::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
        | worth_relational::facade::branch::RelationalBranchBasisDenial::OwnerFailure
        | worth_relational::facade::branch::RelationalBranchBasisDenial::OwnerUnavailable => {
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
        }
    };
    admission_denial(kind, format!("{denial:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_relational_owner_is_an_unavailable_query_basis() {
        let denial = map_basis_denial(
            worth_relational::facade::branch::RelationalBranchBasisDenial::OwnerUnavailable,
        );

        assert_eq!(
            denial.kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
        );
    }
}

pub(in crate::domain_computation::primary_graph::application_query) fn admission_denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph::application_query) fn map_registration_denial(
    denial: super::super::resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    match denial {
        super::super::resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial::Basis(
            denial,
        ) => map_basis_denial(denial),
        super::super::resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial::Snapshot(
            denial,
        ) => {
            let kind = match denial {
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => WorthQueryApplicationQueryAdmissionDenialKind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
                    WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
                }
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                    WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
                }
            };
            admission_denial(kind, format!("snapshot admission denied: {denial:?}"))
        }
    }
}

#[cfg(test)]
#[path = "admission_denial/owner_unavailable.rs"]
mod owner_unavailable_tests;
