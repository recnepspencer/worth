//! What a published host will and will not accept as program meaning.
//!
//! Support is the only source of presentable programs. A revision this host
//! never admitted has no standing to be presented, and a stored activation
//! rendering that answers to no rostered program resolves to nothing, so the
//! commit gate refuses rather than attributing work to a program it cannot
//! name.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::rostered_program_revision;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::unadmitted_program_revision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::program_occurrence::revision_rendering::program_revision_rendering;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_installation::facade::WorthQueryProgramSupportRetirementDenial;

#[test]
fn a_host_presents_only_the_program_revisions_it_admitted() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    assert!(
        support
            .present_for_test(&rostered_program_revision())
            .is_some(),
        "the rostered revision is presentable meaning on this host"
    );
    assert!(
        support
            .present_for_test(&unadmitted_program_revision())
            .is_none(),
        "a revision this host never admitted must not be presentable"
    );
}

#[test]
fn an_activation_rendering_no_rostered_program_answers_to_resolves_to_nothing() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    assert!(
        support
            .rostered_for_rendering_for_test(&program_revision_rendering(
                &rostered_program_revision()
            ))
            .is_some(),
        "the rostered program answers to its own stored rendering"
    );
    assert!(
        support
            .rostered_for_rendering_for_test(&program_revision_rendering(
                &unadmitted_program_revision()
            ))
            .is_none(),
        "an unattributable activation rendering must resolve to no program"
    );
}

#[test]
fn a_host_without_a_published_activation_names_no_occurrence_program() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    assert_eq!(
        support.activation().published_for_test(),
        None,
        "an unseeded branch must not present a permissive default activation"
    );
}

#[test]
fn retained_program_meaning_does_not_authorize_an_inactive_or_replaced_support() {
    let world = installed_authorization_world(true);
    let revision = rostered_program_revision();
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    let prior = support.retain_interpretation_for_test(&revision).unwrap();
    let current = support.retain_interpretation_for_test(&revision).unwrap();
    assert!(prior.same_support_as(&current));

    let replacement = installed_program_support(&(*world.application.installed_schema_for_test()));
    let replacement_pin = replacement
        .retain_interpretation_for_test(&revision)
        .unwrap();
    assert!(!prior.same_support_as(&replacement_pin));

    let retirement = support
        .lifecycle()
        .begin_retirement_for_test(&revision)
        .unwrap();
    assert!(support.retain_interpretation_for_test(&revision).is_none());
    drop(retirement);
    assert!(support.retain_interpretation_for_test(&revision).is_some());
}

#[test]
fn a_commit_names_why_it_cannot_present_a_retiring_or_unadmitted_revision() {
    let world = installed_authorization_world(true);
    let revision = rostered_program_revision();
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    assert!(support.present_for_commit_for_test(&revision).is_ok());
    let unadmitted = support
        .present_for_commit_for_test(&unadmitted_program_revision())
        .err()
        .expect("an unadmitted revision is refused");
    assert_eq!(
        unadmitted.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );

    let retirement = support
        .lifecycle()
        .begin_retirement_for_test(&revision)
        .unwrap();
    let retiring = support
        .present_for_commit_for_test(&revision)
        .err()
        .expect("a retiring revision is refused");
    assert_eq!(
        retiring.kind(),
        WorthQueryApplicationCommitDenialKind::ProgramSupportRetired
    );
    drop(retirement);
    assert!(support.present_for_commit_for_test(&revision).is_ok());
}

#[test]
fn a_completed_retirement_leaves_the_revision_permanently_retired() {
    let world = installed_authorization_world(true);
    let revision = rostered_program_revision();
    let support = installed_program_support(&(*world.application.installed_schema_for_test()));
    let receipt = support
        .lifecycle()
        .begin_retirement_for_test(&revision)
        .unwrap()
        .finish_for_test(0, 0)
        .expect("an unused revision retires");
    assert!(receipt.inventory().can_retire());

    let retired = support
        .present_for_commit_for_test(&revision)
        .err()
        .expect("a retired revision is refused");
    assert_eq!(
        retired.kind(),
        WorthQueryApplicationCommitDenialKind::ProgramSupportRetired
    );
    assert!(support.retain_interpretation_for_test(&revision).is_none());
    assert!(matches!(
        support.lifecycle().begin_retirement_for_test(&revision),
        Err(WorthQueryProgramSupportRetirementDenial::AlreadyRetired { .. })
    ));
}
