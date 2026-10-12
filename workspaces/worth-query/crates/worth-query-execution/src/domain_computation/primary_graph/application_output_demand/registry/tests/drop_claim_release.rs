//! Direct Drop release stays bounded and leaves unvisited debt noted.
use super::*;
fn stopped_ready() -> DemandState {
    let DemandState::Output(mut output) = closed_retirement::ready() else {
        panic!("Ready")
    };
    output.stop(super::super::WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "newer",
    ));
    DemandState::Output(output)
}

#[test]
fn registry_drop_closed_claim_release_is_two_rows_without_a_registry_pass() {
    let mut contacts = Vec::new();
    for neighbors in [1, 64] {
        let registry = super::super::WorthQueryOutputDemandRegistry::default();
        let old: Vec<_> = (1..=5).map(|slot| key("quota-chain", 1, slot)).collect();
        let mut state = registry.state.lock().unwrap();
        for (index, key) in old.iter().enumerate() {
            let mut row = record(occurrence(), stopped_ready(), usize::from(index == 0));
            row.framework_required_count = usize::from(index != 0);
            if let Some(upstream) = old.get(index + 1) {
                row.prerequisites.push(Arc::new(upstream.clone()));
                state.required_reserved_bytes += row.prerequisites.capacity()
                    * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            }
            state.records.insert(key.clone(), row);
            let member = state.prepare_required_member(key).unwrap();
            state.install_required_member(member);
            let newest = super::key("quota-chain", 2, index as u8 + 1);
            state.records.insert(
                newest.clone(),
                record(occurrence(), closed_retirement::ready(), 0),
            );
            let member = state.prepare_required_member(&newest).unwrap();
            state.install_required_member(member);
        }
        for slot in 0..neighbors {
            state.records.insert(
                key("neighbor", 1, 100 + slot),
                record(occurrence(), DemandState::Admitted, 1),
            );
        }
        retain_owned_key(&mut state, &old[0]);
        let wake = Arc::clone(&state.records[&old[0]].wake);
        drop(state);
        let whole = super::super::record_map::whole_rows();
        let occurrence = super::super::record_map::occurrence_rows_visited();
        drop(interest(&registry, old[0].clone(), wake));
        contacts.push(super::super::record_map::occurrence_rows_visited() - occurrence);
        assert_eq!(super::super::record_map::whole_rows(), whole);
        let state = registry.state.lock().unwrap();
        for key in &old[..2] {
            assert!(!state.records[key].has_cached_ready());
            assert!(state.records[key].closed_cleanup.queued());
        }
        assert!(state.records[&old[2]].has_cached_ready());
        assert!(state.records[&old[2]].closed_cleanup.queued());
        assert_eq!(state.records[&old[2]].framework_required_count, 0);
    }
    assert_eq!(contacts[0], contacts[1]);
}
