//! What a published host will and will not accept as program meaning.
//!
//! Support is the only source of presentable programs. A revision this host
//! never admitted has no standing to be presented, and a stored activation
//! rendering that answers to no rostered program resolves to nothing, so the
//! commit gate refuses rather than attributing work to a program it cannot
//! name.

use super::fixture::{
    installed_authorization_world, installed_program_support, rostered_program_revision,
    unadmitted_program_revision,
};
use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;

#[test]
fn a_host_presents_only_the_program_revisions_it_admitted() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&world.application.installed_schema);
    assert!(
        support.present(&rostered_program_revision()).is_some(),
        "the rostered revision is presentable meaning on this host"
    );
    assert!(
        support.present(&unadmitted_program_revision()).is_none(),
        "a revision this host never admitted must not be presentable"
    );
}

#[test]
fn an_activation_rendering_no_rostered_program_answers_to_resolves_to_nothing() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&world.application.installed_schema);
    assert!(
        support
            .rostered_for_rendering(&program_revision_rendering(&rostered_program_revision()))
            .is_some(),
        "the rostered program answers to its own stored rendering"
    );
    assert!(
        support
            .rostered_for_rendering(&program_revision_rendering(&unadmitted_program_revision()))
            .is_none(),
        "an unattributable activation rendering must resolve to no program"
    );
}

#[test]
fn a_host_without_a_published_activation_names_no_occurrence_program() {
    let world = installed_authorization_world(true);
    let support = installed_program_support(&world.application.installed_schema);
    assert_eq!(
        support.activation().published(),
        None,
        "an unseeded branch must not present a permissive default activation"
    );
}

#[test]
fn retained_program_meaning_does_not_authorize_an_inactive_or_replaced_support() {
    let world = installed_authorization_world(true);
    let revision = rostered_program_revision();
    let support = installed_program_support(&world.application.installed_schema);
    let prior = support.retain_interpretation(&revision).unwrap();
    let current = support.retain_interpretation(&revision).unwrap();
    assert!(prior.same_support_as(&current));

    let replacement = installed_program_support(&world.application.installed_schema);
    let replacement_pin = replacement.retain_interpretation(&revision).unwrap();
    assert!(!prior.same_support_as(&replacement_pin));

    let retirement = support.lifecycle().begin_retirement(&revision).unwrap();
    assert!(support.retain_interpretation(&revision).is_none());
    drop(retirement);
    assert!(support.retain_interpretation(&revision).is_some());
}
