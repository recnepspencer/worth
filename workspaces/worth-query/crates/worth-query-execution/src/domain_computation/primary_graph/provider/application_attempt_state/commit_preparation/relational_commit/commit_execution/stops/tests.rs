use super::*;
use crate::domain_computation::{
    WorthQueryProviderSessionCommitStop as Stop, WorthQueryProviderSessionDenialKind as Kind,
};
mod mapping_contracts;

#[test]
fn maintenance_lifetime_and_budgets_keep_their_paths() {
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
    let Stop::Deferred(_) = index_preparation_stop(DerivedIndexMaintenanceDenial {
        kind: Maintenance::CandidateLifetimeExpired {
            maximum_lifetime_millis: 42,
        },
        work: Default::default(),
    }) else {
        panic!("expired candidate must defer")
    };
}

mod native_preparation;
