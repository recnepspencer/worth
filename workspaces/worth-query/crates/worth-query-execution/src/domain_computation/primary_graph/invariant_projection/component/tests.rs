//! Real installed component reads qualify inspection, not ordinary decision scopes.
use super::*;
use std::collections::BTreeSet;
use worth_execution::ExecutionAllocationPolicy as Allocation;
mod fixture;
mod retirement;
use fixture::*;

#[test]
fn installed_components_use_all_source_directions_and_keep_definition_qualified_names() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = install();
    world.application.with_host_advancement(|phase| {
        let completed = world.invariant.project(&phase, |reader| {
            let last = reader.resolve_entity(NodeKey::reference(), 2_u64).unwrap();
            let before = reader.work();
            let shell = reader.component_of(Shells::reference(), &last).unwrap();
            let after = reader.work();
            assert!(after.endpoint_records_read() > before.endpoint_records_read());
            assert!(after.adjacency_lists_read() > before.adjacency_lists_read());
            assert!(after.adjacency_edges_inspected() > before.adjacency_edges_inspected());
            assert_eq!(shell.len(), 3);
            let members = shell.members().collect::<Vec<_>>();
            assert_eq!(reader.work(), after, "borrowed iteration performs no graph acquisitions");
            let explicit_before = reader.work();
            for member in &members {
                reader.relations_from(A::reference(), member).unwrap();
                reader.relations_to(A::reference(), member).unwrap();
                reader.relations_from(B::reference(), member).unwrap();
                reader.relations_to(B::reference(), member).unwrap();
            }
            let explicit_after = reader.work();
            assert_eq!(after.adjacency_lists_read() - before.adjacency_lists_read(),
                explicit_after.adjacency_lists_read() - explicit_before.adjacency_lists_read());
            assert_eq!(after.adjacency_edges_inspected() - before.adjacency_edges_inspected(),
                explicit_after.adjacency_edges_inspected() - explicit_before.adjacency_edges_inspected(),
                "component coupling reads equal explicit physical acquisition in both directions");
            let keys = members.iter().map(|member| reader.field(member, NodeKey::reference()).unwrap())
                .collect::<BTreeSet<_>>();
            assert_eq!(keys, BTreeSet::from([0, 1, 2]));
            let first = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
            let again = reader.component_of(Shells::reference(), &first).unwrap();
            assert_eq!(shell.identity(), again.identity());
            let a_only = reader.component_of(OnlyA::reference(), &last).unwrap();
            assert_eq!(a_only.len(), 1);
            assert_ne!(shell.identity().bytes(), a_only.identity().bytes());
            let singleton = reader.resolve_entity(NodeKey::reference(), 4_u64).unwrap();
            assert_eq!(reader.component_of(Shells::reference(), &singleton).unwrap().len(), 1);
            let self_member = reader.resolve_entity(NodeKey::reference(), 3_u64).unwrap();
            assert_eq!(reader.component_of(Shells::reference(), &self_member).unwrap().len(), 1);
            let lineage = reader.component_lineage(Shells::reference(), shell.identity()).unwrap();
            assert_eq!(lineage.latest_identity().as_ref(), Some(shell.identity()));
            assert!(lineage.retired_at().is_none());
            assert_eq!(lineage.events().len(), 0);
            // A copied descriptive marker cannot rebind an identity to another set.
            let other = ApplicationComponentSetRef::<Schema, Shells, Node>::new("OnlyA", Node::reference());
            assert_eq!(reader.component_lineage(other, shell.identity()).unwrap_err().kind(), Kind::ForeignIdentity);
            shell
        }, Allocation::SystemAllocation).unwrap();
        assert_eq!(completed.output().len(), 3);
    }).unwrap();
    world.application.close_live_delivery();
}

#[test]
fn fresh_issued_snapshot_reselects_components_by_definition_and_foreign_members_refuse() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = install();
    world.application.with_host_advancement(|phase| {
        let prior = world.invariant.project(&phase, |reader| {
            let root = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
            reader.component_of(Shells::reference(), &root).unwrap()
        }, Allocation::SystemAllocation).unwrap();
        let foreign = world.other_authority.project(&phase, |reader|
            reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap(),
            Allocation::SystemAllocation).unwrap();
        let fresh = world.invariant.project(&phase, |reader| {
            assert_eq!(reader.component_of(Shells::reference(), foreign.output()).unwrap_err().kind(), Kind::ForeignIdentity);
            let old = prior.output().aggregate();
            let pin = reader.runtime.read_truth().pin_read_capability(reader.snapshot, &[], reader.execution).unwrap();
            assert_ne!(old.source().snapshot_id(), pin.snapshot().snapshot_id(), "independently issued Native snapshot, not a copied handle");
            assert_eq!(old.source().version_id(), pin.snapshot().version_id());
            let result = reader.execution.run_native_with_checkpoint(|context|
                old.remains_equal_at(&pin, reader.execution, context, None).unwrap()).unwrap();
            assert!(result.0, "the actual definition-qualified reselect matches the admitted product image");
            let missing = ApplicationComponentSetRef::<Schema, Shells, Node>::new("Missing", Node::reference());
            let root = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
            assert_eq!(reader.component_of(missing, &root).unwrap_err().kind(), Kind::ComponentSetNotInstalled);
        }, Allocation::SystemAllocation).unwrap();
        drop(fresh);
    }).unwrap();
    world.application.close_live_delivery();
}
