//! Adoption precedence and empty-home skipping through the public builder.
use super::checkpoint_transition::{
    assert_record_resolves, configuration, home_of, record, source, LegacyTemporalProgram,
};
use super::*;
use application_installation::{
    ApplicationHome, WorthQueryHomeOpening as Opening, WorthQueryOpenAdoption as Adoption,
    WorthQueryOpenAdoptionResources as Resources,
};

#[test]
fn a_rostered_adoption_predecessor_adopts_instead_of_resuming() {
    let (source, predecessor) = source();
    let legacy = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    let mut calls = 0;
    let adopted = application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .roster(application_installation::WorthQueryApplicationProgramRoster::new().support(legacy))
    .adopt_on_open(Adoption::new(
        predecessor.clone(),
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            calls += 1;
            writer.bind_entity(record("rostered-adoption", 127))
        },
    ))
    .open(home_of(&source))
    .expect("adoption wins when its predecessor is also rostered");
    assert_eq!(
        adopted.opening(),
        &Opening::Adopted {
            from: predecessor,
            installed: *validated_program().revision()
        }
    );
    assert_record_resolves(&adopted, 127);
    assert_eq!(calls, 1);
}

#[test]
fn an_empty_home_seeds_and_skips_its_declared_adoption() {
    let (_, predecessor) = source();
    let mut seeds = 0;
    let mut adoptions = 0;
    let started = application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .initial_state(|graph, installed| {
        seeds += 1;
        let principal = installed
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        seed_graph(graph, &principal, "empty-home", 0, 1, true);
        graph.bind_entity(record("empty-seed", 131))
    })
    .adopt_on_open(Adoption::new(
        predecessor,
        Resources::bounded(1, 1, 1).unwrap(),
        |_, _| {
            adoptions += 1;
            Ok(())
        },
    ))
    .open(ApplicationHome::memory())
    .expect("an empty home skips adoption and its preflight");
    assert_eq!(started.opening(), &Opening::Started);
    assert_record_resolves(&started, 131);
    assert_eq!(seeds, 1);
    assert_eq!(adoptions, 0);
}
