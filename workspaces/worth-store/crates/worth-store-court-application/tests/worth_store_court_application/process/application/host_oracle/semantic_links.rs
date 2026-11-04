use super::*;
use host_observation::{comparison, custody, occurrence, projection, requests};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::WorthQueryApplicationHomeLocation;

#[test]
fn semantic_comparison_refuses_wrong_custody_targets_causes_and_native_occurrences() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let hosts: Vec<_> = [
        WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
        WorthQueryApplicationHomeLocation::MemoryOnly,
    ]
    .into_iter()
    .map(|location| {
        let home = host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            None,
            Default::default(),
            &execution,
        );
        home
    })
    .collect();
    let mut map = comparison::HostOracleComparison::default();
    comparison::compare(
        &mut map,
        &projection::observe(hosts[0].runtime(), &execution),
        &projection::observe(hosts[1].runtime(), &execution),
    )
    .unwrap();
    let mut previous_publication = None;
    let mut previous_head = None;
    for (key, value) in [(101, 180), (102, 181)] {
        for home in &hosts {
            assert!(matches!(requests::set(home.runtime(), key, value), worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }));
        }
        let home = projection::observe(hosts[0].runtime(), &execution);
        let reference = projection::observe(hosts[1].runtime(), &execution);
        comparison::compare(&mut map, &home, &reference).unwrap_or_else(|cause| {
            panic!("actual prefix {cause:?}; Home={home:?}; reference={reference:?}")
        });
        if previous_publication.is_none() {
            previous_head = Some(home.head.clone());
            previous_publication = home
                .occurrences
                .iter()
                .find_map(|c| c.relational_publication.clone());
        }
    }
    let home = projection::observe(hosts[0].runtime(), &execution);
    let reference = projection::observe(hosts[1].runtime(), &execution);
    comparison::compare(&mut map, &home, &reference).unwrap_or_else(|cause| {
        panic!("actual prefix {cause:?}; Home={home:?}; reference={reference:?}")
    });
    let expected_history = map.reference_occurrences(&reference.head).unwrap();
    occurrence::compare(map.identities_mut(), &home.occurrences, &expected_history).unwrap();

    let retained = |head: projection::HeadProjection| projection::CustodyProjection {
        owner: head.owner,
        identity: 1,
        effects: Some(1),
        settlement: Some(false),
        cause: Some(custody::CustodyCause::CallerAbandoned),
        expected: Some(head),
        actions: Some(vec![custody::CustodyAction::Inspect]),
        original_effect: None,
    };
    let good = retained(home.head.clone());
    let expected = retained(reference.head.clone());
    custody::compare(map.identities_mut(), &[good.clone()], &[expected.clone()]).unwrap();
    let mut wrong = good.clone();
    wrong.cause = Some(custody::CustodyCause::StaleProductHead);
    assert!(custody::compare(map.identities_mut(), &[wrong], &[expected.clone()]).is_err());
    let mut wrong = good.clone();
    wrong.actions = Some(vec![custody::CustodyAction::CloseOwner]);
    assert!(custody::compare(map.identities_mut(), &[wrong], &[expected.clone()]).is_err());
    let mut wrong = good.clone();
    wrong.expected.as_mut().unwrap().branch = "wrong-semantic-branch".into();
    assert!(custody::compare(map.identities_mut(), &[wrong], &[expected.clone()]).is_err());
    let mut wrong = good;
    let previous = previous_head.unwrap();
    assert_ne!(previous.commit, home.head.commit);
    wrong.expected.as_mut().unwrap().commit = previous.commit;
    assert!(custody::compare(map.identities_mut(), &[wrong], &[expected]).is_err());

    let mut wrong = home.occurrences.clone();
    wrong[0].relational_changed = !wrong[0].relational_changed;
    assert!(occurrence::compare(map.identities_mut(), &wrong, &expected_history).is_err());
    let (latest_index, latest) = home
        .occurrences
        .iter()
        .enumerate()
        .find_map(|(index, c)| c.relational_publication.as_ref().map(|p| (index, p)))
        .unwrap();
    let previous = previous_publication.unwrap();
    assert_ne!(
        latest, &previous,
        "two real mutations published two Native facts"
    );
    let mut wrong = home.occurrences.clone();
    wrong[latest_index].relational_publication = Some(previous);
    assert!(occurrence::compare(map.identities_mut(), &wrong, &expected_history).is_err());
    let mut wrong: projection::StateProjection =
        serde_json::from_value(serde_json::to_value(&home).unwrap()).unwrap();
    wrong.history_crossed_retired = !wrong.history_crossed_retired;
    assert!(comparison::compare(&mut map, &wrong, &reference).is_err());
    let mut missing: projection::StateProjection =
        serde_json::from_value(serde_json::to_value(&home).unwrap()).unwrap();
    missing.occurrences.pop();
    assert!(
        comparison::compare(&mut map, &missing, &reference).is_err(),
        "a cached Home projection cannot supply a missing current occurrence"
    );

    use host_observation::reference_history::ReferenceOccurrencePrefix;
    let mut prefix = ReferenceOccurrencePrefix::default();
    prefix.observe(&expected_history).unwrap();
    assert_eq!(prefix.trace(&reference.head).unwrap(), expected_history);
    let mut changed = expected_history[0].clone();
    changed.relational_changed = !changed.relational_changed;
    assert!(prefix.observe(&[changed]).is_err());
    let mut missing_parent = ReferenceOccurrencePrefix::default();
    missing_parent.observe(&expected_history[..1]).unwrap();
    assert!(missing_parent.trace(&reference.head).is_err());
    let mut cycle = expected_history[0].clone();
    cycle.parent = Some(cycle.commit);
    let mut cyclic = ReferenceOccurrencePrefix::default();
    cyclic.observe(&[cycle]).unwrap();
    assert!(cyclic.trace(&reference.head).is_err());
}
