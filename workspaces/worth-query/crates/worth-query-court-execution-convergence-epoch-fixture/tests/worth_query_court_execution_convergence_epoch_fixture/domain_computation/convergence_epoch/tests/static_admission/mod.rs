use worth_query_admission::facade::convergence_epoch::admit_convergence_epoch_contract;
use worth_query_admission::facade::convergence_epoch::WorthQueryConvergenceAdmissionDenialKind;

use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::FixtureConvergenceContract;
use crate::domain_computation::convergence_epoch::tests::fixture::static_installed_world::static_convergence_admission_fixture;
use crate::domain_computation::convergence_epoch::tests::fixture::static_installed_world::StaticConvergenceAdmissionFixture;

#[test]
fn exact_installed_contract_admission_seals_every_static_basis_once() {
    let fixture = static_convergence_admission_fixture(FixtureConvergenceContract::Bounded);
    let admitted = admit_convergence_epoch_contract(&fixture.operation, fixture.artifact)
        .expect("exact installed convergence authorities must admit");
    let counters = admitted.counters();
    assert_eq!(counters.installed_authority_check_count(), 1);
    assert_eq!(counters.operation_evidence_check_count(), 1);
    assert_eq!(counters.convergence_contract_check_count(), 1);
    assert_eq!(admitted.iteration_bound(), 3);
}

#[test]
fn invalid_static_contract_classes_deny_and_return_the_exact_artifact() {
    let cases = [
        (
            FixtureConvergenceContract::NonIterative,
            WorthQueryConvergenceAdmissionDenialKind::NonIterativeContract,
        ),
        (
            FixtureConvergenceContract::MissingSearch,
            WorthQueryConvergenceAdmissionDenialKind::MissingCandidateSearch,
        ),
    ];
    for (contract, expected) in cases {
        let fixture = static_convergence_admission_fixture(contract);
        let artifact_identity = fixture.artifact.admission_identity().to_owned();
        let rejection = match admit_convergence_epoch_contract(&fixture.operation, fixture.artifact)
        {
            Ok(_) => panic!("invalid static convergence class entered admission"),
            Err(rejection) => rejection,
        };
        assert_eq!(rejection.denial().kind(), expected);
        assert_eq!(
            rejection.into_artifact().admission_identity(),
            &artifact_identity
        );
    }
}

#[test]
fn foreign_installed_authority_denial_preserves_artifact_for_its_real_operation() {
    let first = bounded_fixture();
    let second = bounded_fixture();
    let foreign_artifact_identity = second.artifact.admission_identity().to_owned();
    let rejection = match admit_convergence_epoch_contract(&first.operation, second.artifact) {
        Ok(_) => panic!("foreign installed artifact entered static convergence admission"),
        Err(rejection) => rejection,
    };
    assert_eq!(
        rejection.denial().kind(),
        WorthQueryConvergenceAdmissionDenialKind::ForeignInstalledAuthorities
    );
    let recovered = rejection.into_artifact();
    assert_eq!(recovered.admission_identity(), &foreign_artifact_identity);
    admit_convergence_epoch_contract(&second.operation, recovered)
        .expect("recovered artifact must still admit with its exact installed operation");
    admit_convergence_epoch_contract(&first.operation, first.artifact)
        .expect("unrelated exact static authority must remain unaffected");
}

pub fn bounded_fixture() -> StaticConvergenceAdmissionFixture {
    static_convergence_admission_fixture(FixtureConvergenceContract::Bounded)
}
