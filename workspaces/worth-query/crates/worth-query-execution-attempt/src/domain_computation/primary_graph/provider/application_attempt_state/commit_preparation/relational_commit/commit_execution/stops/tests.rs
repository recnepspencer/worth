use super::*;
use crate::domain_computation::{
    WorthQueryProviderSessionCommitStop as Stop, WorthQueryProviderSessionDenialKind as Kind,
};
pub mod mapping_contracts;

#[test]
fn maintenance_budgets_identity_and_rejection_keep_their_paths() {
    use worth_relational::facade::indexes::{
        DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind as Maintenance,
    };
    for (kind, expected) in [
        (
            Maintenance::WorkBudgetExceeded,
            Kind::IndexMaintenanceBudgetExceeded,
        ),
        (
            Maintenance::ColdReconstructionRequired,
            Kind::IndexMaintenanceBudgetExceeded,
        ),
        (
            Maintenance::GenerationIdentityExhausted,
            Kind::IndexGenerationIdentityExhausted,
        ),
        (Maintenance::CommitMismatch, Kind::ProviderRejected),
    ] {
        let Stop::PreEffectDenied(failure) =
            index_preparation_stop(DerivedIndexMaintenanceDenial {
                kind,
                work: Default::default(),
            })
        else {
            panic!("wrong maintenance stop")
        };
        assert_eq!(failure.kind(), expected);
    }
}

pub mod native_preparation;
