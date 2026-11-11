//! Distinct extras sharing one aspect map each report their own declared bound.
use super::*;
use crate::storage::substrate::StorageAllocationExtent;

#[test]
fn extras_sharing_one_aspect_map_each_report_the_checked_per_holder_bound() {
    let (_, source) = patch("shared aspect map");
    let TransitionOutcome::Success(Some(state)) = source.apply_to_optional(None) else {
        panic!("admitted source state")
    };
    let state = state.into_parts().into_parts().0;
    // Two independent carriers hold clones of one persistent aspect map.
    let original = EntityExtra::from_unbudgeted_parts(None, None, Some(state.clone()));
    let copy = EntityExtra::from_unbudgeted_parts(None, None, Some(state.clone()));
    let key = AspectKey::new("data").unwrap();
    let held = |extra: &EntityExtra| {
        let state = extra.authoritative_aspect_state().unwrap();
        state.get(&key).unwrap().key().as_str().as_ptr()
    };
    assert_eq!(
        held(&original),
        held(&copy),
        "the carriers share the admitted aspect backing instead of duplicating it"
    );
    let map_bound = u64::try_from(state.aspects().retained_storage_bytes().unwrap()).unwrap();
    assert!(map_bound > 0);
    for extra in [&original, &copy] {
        assert_eq!(
            EntityRecordKind::extra_reachable_bound_bytes(extra),
            Some(map_bound)
        );
    }
    let layout = StorageAllocation::<EntityExtra>::layout_bytes();

    let mut current = arena(original);
    let first = current.retained_extra(0).unwrap().id();
    current.apply_extra_update(0, copy, VersionId(2)).unwrap();
    let second = current.retained_extra(0).unwrap().id();
    assert_ne!(first, second, "two distinct carriers hold the shared map");
    let mut observed = Observed::default();
    current.visit_authoritative_allocations(false, &mut observed);
    let carriers: Vec<_> = observed
        .0
        .iter()
        .filter(|entry| entry.id == first || entry.id == second)
        .collect();
    assert_eq!(carriers.len(), 2, "each carrier is one inventory row");
    for carrier in &carriers {
        assert_eq!(carrier.extent, StorageAllocationExtent::Checked);
        assert_eq!(carrier.bytes, layout + map_bound);
    }
    // One holder's bound already covers the whole shared backing, so the two
    // rows together overbound it by one further map bound; never an undercount.
    let reported: u64 = carriers.iter().map(|carrier| carrier.bytes).sum();
    assert_eq!(reported, 2 * layout + 2 * map_bound);
    assert!(reported > 2 * layout + map_bound);
}
