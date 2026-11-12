//! Owner field deltas and member lifetimes share the declared live-owner inverse.
use super::*;

#[test]
fn a_live_owner_field_and_deleted_members_reverse_through_one_native_preimage() {
    for (remove, value) in [(false, 99), (true, 99), (false, 7)] {
        let host = world::install(world::Fault::None, Hashes::default());
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let mut graph = image(3);
        for row in &mut graph.members {
            row.next = None;
        }
        created(&host, &request, graph);
        let before = local_snapshot(&request);
        assert_eq!(before.value, 7);
        let mut input = selection(if remove { &["part-1"] } else { &[] }, None, &[]);
        input.owner_value = Some(value);
        let receipt = delete(&host, &request, input, "change-owner-and-members");
        let changed = local_snapshot(&request);
        assert_eq!(changed.id, before.id);
        assert_eq!(changed.value, value);
        assert_eq!(changed.members.len(), if remove { 2 } else { 3 });
        reverse(&request, &receipt, "restore-owner-and-members");
        let after = local_snapshot(&request);
        assert_eq!(after.id, before.id);
        assert_eq!(after.value, 7);
        assert_eq!(after.members.len(), 3);
        assert_eq!(after.members[0], before.members[0]);
        assert_eq!(after.members[2], before.members[2]);
        assert_eq!(after.members[1].1, before.members[1].1);
        if remove {
            assert_ne!(after.members[1].0, before.members[1].0);
        } else {
            assert_eq!(after.members[1].0, before.members[1].0);
        }
    }
}
