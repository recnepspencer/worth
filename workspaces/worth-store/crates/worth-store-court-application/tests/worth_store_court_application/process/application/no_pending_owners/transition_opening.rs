//! Open a genuine predecessor, close it, then admit its declared successor.
use super::*;
use worth_query_testkit_installation::facade::installation::application_home_binding as bindings;

// These courts stop at intent admission. The catalog is descriptive protocol
// DATA; no Native target-inventory authority is claimed by this fixture.
pub(super) fn target_meaning() -> worth_foundational::facade::HomeProgramTransitionTargetMeaning {
    worth_foundational::facade::HomeProgramTransitionTargetMeaning::from_recorded(
        0,
        worth_foundational::facade::HomeProgramTransitionCatalogDigest::from_recorded([9; 32]),
    )
}

pub(super) fn successor_request(
    location: &std::path::Path,
) -> worth_store_application::ApplicationHomeRequest {
    let native = || crate::workload::runtime_builder(crate::schema_transition::SOURCE).resolve();
    worth_store_application::ApplicationHomeRequest::new(
        location.into(),
        native(),
        bindings::admitted_transition_successor_binding(),
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-4111-8111-111111111111",
        )
        .unwrap(),
    )
    .with_program_transition_predecessor(native(), bindings::admitted_home_binding())
}

pub(super) fn open_transition(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    {
        let predecessor = open(location, execution);
        drop(predecessor.bootstrap_root(execution));
        close(predecessor.home, execution);
    }
    worth_store_testkit_application_oracle::world_home::open(successor_request(location), execution)
        .expect("the same Home readmits the genuine declared predecessor before transition")
}
