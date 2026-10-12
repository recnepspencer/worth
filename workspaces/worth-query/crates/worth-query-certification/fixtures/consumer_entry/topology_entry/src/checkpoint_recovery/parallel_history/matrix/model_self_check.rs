//! Negative accounting models must fail the same law assertion as a wrong driver.
use super::*;
fn charges(actual: (u64, u64), declared: &World) {
    let work = declared
        .members
        .iter()
        .flat_map(|m| &m.partitions)
        .map(|p| p.work + super::super::structural_cost::reduction_work(&[1]))
        .sum::<u64>();
    let span = declared
        .members
        .iter()
        .map(|m| {
            m.partitions
                .iter()
                .map(|p| p.work + super::super::structural_cost::reduction_work(&[1]))
                .sum::<u64>()
        })
        .max()
        .unwrap();
    assert_eq!(
        actual,
        (work, span),
        "a dispatched failed wave charges whole compute and takes its critical path"
    );
}
#[test]
fn a_prefix_only_model_and_a_summed_span_are_convicted() {
    let world = seeded_world::world(seeded_world::Family::Unequal, SEED);
    let ranks = world
        .members
        .iter()
        .enumerate()
        .map(|(i, m)| (m.key, i))
        .collect();
    let waves = model::waves(&world, &ranks);
    let failure = Some((world.members[1].key, control::Phase::Complete));
    let actual = model::advancement(&world, &waves, failure);
    charges((actual.work, actual.span), &world);
    let prefix = world.members[..1]
        .iter()
        .flat_map(|m| &m.partitions)
        .map(|p| p.work + super::super::structural_cost::reduction_work(&[1]))
        .sum::<u64>();
    assert!(
        std::panic::catch_unwind(|| charges((prefix, actual.span), &world)).is_err(),
        "prefix-only mutant model is red at the charge law"
    );
    assert!(
        std::panic::catch_unwind(|| charges((actual.work, actual.work), &world)).is_err(),
        "summed-span mutant model is red at the span law"
    );
}
