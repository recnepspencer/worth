//! A real AABB false positive and endpoint edit exercise distinct exact watch truth.
use super::super::refinement;
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
#[test]
fn exact_refinement_segment_contact_uses_related_points_and_candidate_stable_watch() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::exact_refinement_segment_contact_uses_related_points_and_candidate_stable_watch"
        ),
    );
}
fn run() {
    let calls = Arc::new(provider::Calls::default());
    let prefixes = Arc::new(prefix_observation::Calls::default());
    let world = host::install_geometry(
        prefixes.clone(),
        Arc::new(prefix_rule::Calls::new(prefixes)),
        Configuration::new(calls),
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    for (offset, edit) in [
        Edit::Vertex {
            directory: "catalog".into(),
            identity: "exact-a".into(),
            x: 0,
            y: 0,
        },
        Edit::Vertex {
            directory: "catalog".into(),
            identity: "exact-b".into(),
            x: 10,
            y: 10,
        },
        Edit::Edge {
            directory: "A".into(),
            identity: "exact-edge".into(),
            start: "exact-a".into(),
            end: "exact-b".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        commit(
            &world,
            &principal,
            &scope,
            edit,
            0x9199_3000 + offset as u64,
        );
    }
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let probe = Coordinates::new(4, 5, 4, 5);
    let broad = || query::FindBoxes {
        directory: "A".into(),
        probe,
    };
    let exact = || refinement::FindContact {
        directory: "A".into(),
        probe,
    };
    let (candidate, mut candidate_watch) = request
        .query(broad())
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    assert_eq!(
        candidate.rows().len(),
        1,
        "probe lies inside genuine edge AABB"
    );
    let (refined, mut exact_watch) = request
        .query(exact())
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    assert!(
        refined.rows().is_empty(),
        "point (4,5) is off the diagonal segment"
    );
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "catalog".into(),
            vertex: "exact-b".into(),
            x: 10,
            y: 10,
        },
        0x9199_300f,
    );
    match request
        .query(broad())
        .next_watch(&mut candidate_watch)
        .unwrap()
    {
        WorthQueryManagedIndexWatchOutcome::Pending => (),
        WorthQueryManagedIndexWatchOutcome::Delivered(update) => {
            assert_eq!(values(update.read().rows()), values(candidate.rows()))
        }
        _ => panic!("same endpoint value must preserve supported watch availability"),
    }
    assert!(matches!(
        request
            .query(broad())
            .next_watch(&mut candidate_watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(matches!(
        request.query(exact()).next_watch(&mut exact_watch).unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert_eq!(
        values(request.query(broad()).execute().unwrap().rows()),
        values(candidate.rows())
    );
    assert!(request.query(exact()).execute().unwrap().rows().is_empty());
    for (offset, (x, y, count)) in [(4, 5, 1), (10, 10, 0)].into_iter().enumerate() {
        commit(
            &world,
            &principal,
            &scope,
            Edit::Move {
                directory: "catalog".into(),
                vertex: "exact-b".into(),
                x,
                y,
            },
            0x9199_3010 + offset as u64,
        );
        let WorthQueryManagedIndexWatchOutcome::Delivered(candidate) = request
            .query(broad())
            .next_watch(&mut candidate_watch)
            .unwrap()
        else {
            panic!("candidate remains true with changed endpoint output")
        };
        assert_eq!(candidate.read().rows().len(), 1);
        let WorthQueryManagedIndexWatchOutcome::Delivered(refined) =
            request.query(exact()).next_watch(&mut exact_watch).unwrap()
        else {
            panic!("closed exact contact changed while candidate stayed true")
        };
        assert_eq!(refined.read().rows().len(), count);
    }
    assert!(matches!(
        request.query(exact()).next_watch(&mut exact_watch).unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
}
