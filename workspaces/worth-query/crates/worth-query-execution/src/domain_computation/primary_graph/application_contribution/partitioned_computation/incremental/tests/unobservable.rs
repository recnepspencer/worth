//! A retained key the next attempt cannot observe marks the partitions that
//! read it, as a moved fact does, and denies nothing.

use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey as Key;

#[test]
fn an_unobservable_key_gathers_its_partition_again_and_denies_nothing() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let outcome = *first.outcome.as_ref().unwrap();
    let mut state = first.sealed.unwrap().unwrap().state;
    let key = state
        .facts
        .facts()
        .find(|(_, _, readers)| readers.partitions().len() == 1)
        .map(|(key, _, _)| key.clone())
        .expect("one gather reads the status");
    let Key::Field {
        entity_id, locator, ..
    } = key.clone()
    else {
        panic!("the status is a field's key: {key:?}");
    };
    // No entity of this name is declared, so the key observes as an error.
    let undeclared = Key::Field {
        entity: "Undeclared".to_owned(),
        entity_id,
        locator,
    };
    state.facts.rename_key(&key, undeclared);
    let prior = ComputationPrior::new(edition(), Ok(Arc::new(state)), None);

    let next = attempt(&world, &installed, Some(prior));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert_eq!(next.gathered, [1], "only the partition that read it");
    assert_eq!(next.outcome.unwrap(), outcome);
    assert!(
        next.sealed.unwrap().is_some(),
        "the attempt seals and retains"
    );
}
