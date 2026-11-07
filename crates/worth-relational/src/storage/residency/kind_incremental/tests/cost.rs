//! Fixed edits at independent scales; observe the production engine's quotes.
use super::*;
use worth_testkit_relational::maintenance::assert_change_bounded;

fn work(records: usize, history: usize) -> u64 {
    let mut partition = fixture::partition(records);
    for step in 0..history {
        fixture::edit(&mut partition, 0, step);
    }
    // Lifecycle and endpoints repeat every 15 steps. Establish the same live
    // starting point, then measure the same retarget at every history scale.
    let base_step = history.div_ceil(15) * 15;
    fixture::edit(&mut partition, 0, base_step);
    crate::storage::residency::rebuild_region_history_kind_buckets(&mut partition).unwrap();
    let prior = Arc::new(cold(&partition));
    let previous = partition.clone();
    let journal = fixture::edit(&mut partition, 0, base_step + 2);
    let mut charged = 0;
    let updated = update_value(
        &mut partition,
        &previous,
        &prior,
        &journal,
        &mut |units, _| {
            charged += units;
            Ok(())
        },
        &mut |_, _| Ok(None),
    )
    .unwrap();
    assert_eq!(updated.commitment(), cold(&partition).commitment());
    charged
}

#[test]
fn fixed_kind_edits_do_not_grow_with_model_or_retained_history() {
    let base = work(512, 8);
    let model = work(1024, 8);
    let history = work(8, 8192);
    let longer_history = work(8, 16384);
    eprintln!(
        "kind commitment charged work: model {base}->{model}; history {history}->{longer_history}"
    );
    assert_change_bounded("kind commitment", "model", base, model);
    assert_change_bounded("kind commitment", "history", history, longer_history);
}
