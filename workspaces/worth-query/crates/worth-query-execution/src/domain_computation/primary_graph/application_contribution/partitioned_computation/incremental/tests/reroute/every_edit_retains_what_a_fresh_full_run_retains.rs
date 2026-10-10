//! every edit retains what a fresh full run retains.
use super::*;

#[test]
fn every_edit_retains_what_a_fresh_full_run_retains() {
    let world = installed_authorization_world(true);
    let mut applied = std::collections::BTreeSet::new();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            for seed in [0x9e37_79b9_7f4a_7c15_u64, 7, 0x00c0_ffee, 31_337] {
                let installed =
                    WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
                        Rerouted::default(),
                        ComputationRetention::ProducerOperation,
                    );
                let mut seeded = Seeded(seed);
                let mut entries = (1..=6)
                    .map(|id| (id, (id % 3, 10 * id)))
                    .collect::<BTreeMap<_, _>>();
                *installed.owner.entries.lock().unwrap() = entries.clone();
                let fresh = || Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None));
                let mut last = attempt(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &installed,
                    fresh(),
                )
                .sealed
                .unwrap()
                .unwrap();
                for step in 0..12 {
                    let edit = seeded.pick(&EDITS).unwrap();
                    if !apply(edit, &mut entries, &mut seeded) {
                        continue;
                    }
                    applied.insert(edit);
                    let before = std::mem::replace(
                        &mut *installed.owner.entries.lock().unwrap(),
                        entries.clone(),
                    );
                    // Every fourth step moves the label every key read, so every item
                    // is keyed again; otherwise only the items that are new or whose
                    // key or value changed are.
                    let keys = step % 4 == 3;
                    let touched = entries
                        .iter()
                        .filter(|(id, entry)| keys || before.get(id) != Some(entry));
                    let touched = touched.map(|(id, _)| *id).collect::<Vec<_>>();
                    installed.owner.keyed.lock().unwrap().clear();
                    let next = attempt(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                        &world,
                        &installed,
                        Some(moved_facts(last, keys)),
                    );
                    let keyed = std::mem::take(&mut *installed.owner.keyed.lock().unwrap());
                    let full = attempt(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                        &world,
                        &installed,
                        fresh(),
                    );
                    let context =
                        format!("seed {seed:#x}, step {step}, {edit:?}, items {entries:?}");
                    assert!(
                        matches!(next.runs.as_slice(), [(Run::Incremental, None)]),
                        "{context}"
                    );
                    assert_eq!(keyed, touched, "{context}: only touched items are keyed");
                    if edit == Edit::Unchanged {
                        assert!(next.gathered.is_empty(), "{context}");
                    }
                    assert_eq!(next.outcome, full.outcome, "{context}");
                    let next = next.sealed.unwrap().expect("the next run retains");
                    let full = full.sealed.unwrap().expect("the full run retains");
                    assert_eq!(retained(&next), retained(&full), "{context}");
                    last = next;
                }
            }
            assert_eq!(applied, EDITS.into_iter().collect(), "every edit ran");
        })
        .expect("the fixture block owns its installed host call")
}
