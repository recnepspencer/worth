//! Drop keeps the old claim until an admission pays for its transfer.
use super::*;
mod differential;

#[test]
fn registry_drop_cascade_transfers_before_newest_owner_can_close() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", 1, 1, 10);
    let newest = key_with_identity("upstream", 2, 1, 11);
    let dependent = key("dependent", 1, 2);
    let ready = closed_retirement::ready();
    let mut state = registry.state.lock().unwrap();
    state.records.insert(
        old.clone(),
        DemandRecord {
            framework_required_count: 1,
            ..record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "newer row",
                )),
                0,
            )
        },
    );
    let next_row = record(occurrence(), closed_retirement::ready(), 1);
    let next_wake = Arc::clone(&next_row.wake);
    state.records.insert(newest.clone(), next_row);
    let dependent_row = DemandRecord {
        prerequisites: vec![Arc::new(old.clone())],
        ..record(occurrence(), ready, 1)
    };
    let wake = Arc::clone(&dependent_row.wake);
    state.records.insert(dependent.clone(), dependent_row);
    state.required_keys.insert(Arc::new(newest.clone()));
    retain_owned_key(&mut state, &dependent);
    retain_owned_key(&mut state, &newest);
    drop(state);
    let next_owner = interest(&registry, newest.clone(), next_wake);
    drop(interest(&registry, dependent.clone(), wake));
    // The stale claim keeps the newest member through both Drops.
    drop(next_owner);
    let state = registry.state.lock().unwrap();
    assert!(
        state.records.contains_key(&old),
        "Drop must leave the claim transfer unpaid"
    );
    assert_eq!(state.records[&dependent].prerequisites[0].as_ref(), &old);
    assert!(state.records[&dependent].closed_cleanup.queued());
    drop(state);
    registry
        .drain_closed_cleanup_admitted(&mut record_admission())
        .unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&old));
    assert_eq!(state.records[&newest].framework_required_count, 1);
    assert!(state.required_keys.contains(&newest));
    assert_eq!(state.records[&dependent].prerequisites[0].as_ref(), &newest);
}

#[test]
fn registry_drop_cascade_names_its_released_row_bound() {
    let receipts = walk_bounds::receipt_batch();
    let mut contacts = Vec::new();
    for length in [1, 2, 4] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut state = registry.state.lock().unwrap();
        walk_bounds::add_live_custody(&mut state, &receipts, 3);
        state.note_custody_changes(|_, entry| {
            entry.token_count = 1;
            false
        });
        for slot in 1..=length {
            let mut row = record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "closed chain",
                )),
                usize::from(slot == 1),
            );
            row.framework_required_count = usize::from(slot != 1);
            if slot == 1 {
                row.prerequisites
                    .extend((2..=length).map(|upstream| Arc::new(key("chain", 1, upstream))));
            }
            state.records.insert(key("chain", 1, slot), row);
        }
        let root = key("chain", 1, 1);
        retain_owned_key(&mut state, &root);
        let wake = Arc::clone(&state.records[&root].wake);
        drop(state);
        let before = super::super::record_map::occurrence_rows_visited();
        drop(interest(&registry, root, wake));
        contacts.push(super::super::record_map::occurrence_rows_visited() - before);
        registry
            .drain_closed_cleanup_admitted(&mut record_admission())
            .unwrap();
        assert_eq!(
            registry.state.lock().unwrap().records.len(),
            receipts.len() * 3
        );
    }
    // Drop only searches its own one-row occurrence, regardless of its edges.
    assert_eq!(contacts, [1, 1, 1]);
}

#[cfg(feature = "test-query-execution-observer")]
#[test]
fn registry_drop_cascade_names_its_whole_map_claimant_bound() {
    let receipts = walk_bounds::receipt_batch();
    let mut contacts = Vec::new();
    for neighbors in [1, 64] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let old = key_with_identity("upstream", 1, 1, 10);
        let newest = key_with_identity("upstream", 2, 1, 11);
        let dependent = key("dependent", 1, 2);
        let mut state = registry.state.lock().unwrap();
        walk_bounds::add_custody(&mut state, &receipts, 1);
        for slot in 0..neighbors {
            state.records.insert(
                key("neighbor", 1, 20 + slot),
                record(occurrence(), DemandState::Admitted, 1),
            );
        }
        state.records.insert(
            old.clone(),
            DemandRecord {
                framework_required_count: 1,
                ..record(
                    occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "newer",
                    )),
                    0,
                )
            },
        );
        state.records.insert(
            newest.clone(),
            record(occurrence(), closed_retirement::ready(), 1),
        );
        state.required_keys.insert(Arc::new(newest));
        let row = DemandRecord {
            prerequisites: vec![Arc::new(old)],
            ..record(occurrence(), closed_retirement::ready(), 1)
        };
        let wake = Arc::clone(&row.wake);
        state.records.insert(dependent.clone(), row);
        retain_owned_key(&mut state, &dependent);
        drop(state);
        let whole = super::super::record_map::whole_rows();
        let occurrence = super::super::record_map::occurrence_rows_visited();
        drop(interest(&registry, dependent, wake));
        contacts.push((
            super::super::record_map::whole_rows() - whole,
            super::super::record_map::occurrence_rows_visited() - occurrence,
        ));
    }
    // One replaced-row scan and one member scan of the dependent's own row;
    // linking its cue makes no claimant pass or upstream walk.
    assert_eq!(contacts, [(0, 2), (0, 2)]);
}
