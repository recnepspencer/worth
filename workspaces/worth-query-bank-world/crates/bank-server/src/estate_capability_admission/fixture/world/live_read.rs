//! A real Bank world with an owner and a teller for live read custody probes.
use super::*;

pub(in crate::estate_capability_admission::fixture) fn install_live_read_world(
    scenario: &str,
) -> InstalledFixtureWorld {
    let spec = FixtureWorldSpec {
        scenario,
        spec: GrantSpec::view(),
        case_stage: bank_domain::estate::EstateWorkflowStage::Administration,
        specialist_holds_authority: false,
        unrelated_grants: 0,
        composition: FixtureWorldComposition::Admission,
        alternate_emergency_bound: None,
    };
    let identities = seed::identities(scenario);
    let snapshot = snapshot_fixture::live_snapshot();
    let estate_world = compose_estate(&spec);
    let seed = seed::assemble(snapshot, estate_world.clone(), &identities, &spec).employee(
        crate::BankEmployeeAssignmentSeed::new(
            bank_domain::model::EmployeeAssignmentId::new(10_001).unwrap(),
            crate::estate_capability_admission::fixture::INSTITUTION,
            SPECIALIST,
            bank_domain::model::EmployeeRole::Teller,
        ),
    );
    InstalledFixtureWorld {
        runtime: install_runtime(seed, None),
        estate_world,
        identities,
    }
}
