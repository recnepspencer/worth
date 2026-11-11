//! Genuine public maintenance without a rule referencing this index.
use super::super::super::super::super::authentication;
use super::super::super::{host, rule as prefix_rule};
use super::super::{
    authoring::Edit,
    court::{commit, expected, setup, values},
    Configuration,
};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
};
mod cancellation;

fn find(directory: &str, x: i64) -> FindPublicBoxes {
    FindPublicBoxes {
        directory: directory.into(),
        probe: Coordinates::new(x, 0, x, 0),
    }
}
fn all(directory: &str) -> FindPublicBoxes {
    FindPublicBoxes {
        directory: directory.into(),
        probe: Coordinates::new(-1, -1, 100, 1),
    }
}

#[test]
fn public_requirement_prepares_distinct_source_without_invariant_and_refuses_same_head_pinned() {
    let declaration =
        <Catalog as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
            .unwrap();
    assert!(!declaration
        .erased()
        .members()
        .iter()
        .any(|member| matches!(member,
        ApplicationSchemaMember::InvariantManagedIndexSource { requirement, .. }
            if requirement.index() == PublicBoxes::NAME)));
    let public = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::ManagedIndex { definition }
                if definition.parts().name == PublicBoxes::NAME =>
            {
                Some(definition)
            }
            _ => None,
        })
        .expect("the actual installed public definition");
    assert_eq!(public.parts().semantic_version, 5);
    assert!(public.refinement().is_none());
    let configuration = Configuration::default();
    let calls = Arc::clone(&configuration.public_source);
    let prefix_calls = Arc::new(prefix_observation::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(prefix_calls, prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    let builds = calls.builds.load(Ordering::SeqCst);
    assert!(
        builds > 0,
        "the distinct public provider was initialized before any Parent read"
    );
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let original = request.query(all("A")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", 10, all("A").probe));
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    let same_head = request.retain_current_read().unwrap();
    let probes = calls.probe_validations.load(Ordering::SeqCst);
    let error = match request.at(&same_head).query(all("A")).execute() {
        Ok(_) => panic!("an explicit retained read cannot borrow current-only support"),
        Err(error) => error,
    };
    let WorthQueryApplicationRequestQueryDenial::Admission(denial) = error else {
        panic!("basis admission must refuse before managed execution: {error:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported
    );
    assert_eq!(calls.probe_validations.load(Ordering::SeqCst), probes);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        same_head.selected_commit()
    );
    let updates = calls.updates.load(Ordering::SeqCst);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 20,
            y: 0,
        },
        0x919520,
    );
    assert!(calls.updates.load(Ordering::SeqCst) > updates);
    assert_eq!(values(original.rows()), expected("A", 10, all("A").probe));
    assert_eq!(
        values(request.query(find("A", 15)).execute().unwrap().rows()),
        expected("A", 20, find("A", 15).probe)
    );
    // B was shared and maintained without any public B Parent being opened.
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 20, all("B").probe)
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "both scopes attach published roots"
    );
    cancellation::run(&world, &principal, &calls);
}
