use super::routing::ROUTED;
use super::*;
use worth_proof::TransitionOutcome;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;
use worth_runtime_bridge::facade::{RelationalCommittedPatchRequest, TruthCommitIdentity};

#[test]
fn committed_patch_delivery_changes_the_installed_graph_before_world_sealing() {
    let records = CargoRecords::install(true);
    let mut graph = SignalGraph::new();
    let nodes = RouteGraph::install(&mut graph);
    let (bridge, installed, source) = correspondence::install(&records, &mut graph, nodes);
    let mut oracle = oracle::CompositeWorldOracle::bootstrap();
    // Both commits are real FieldSet patches in the same owner and contract.
    // Only grain matches the installed record correspondence.
    for (name, amount, expected_seeds) in [("steel", "5", 0), ("grain", "6", 1)] {
        let basis = records
            .runtime
            .observe_branch(&records.runtime.main_branch_identity())
            .unwrap()
            .1;
        let candidate = records.candidate(&basis, name, amount);
        let RelationalPublicationOutcome::Performed(performed) = records
            .runtime
            .owner_component_services()
            .publication_port()
            .compare_and_publish(candidate)
        else {
            panic!("real source commit must perform")
        };
        let commit = records
            .runtime
            .owner_component_services()
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        let selected = source
            .observe_branch_basis(&records.runtime.main_branch_identity())
            .unwrap()
            .1;
        let selected_lease = source.retain_branch_basis_for_bridge(&selected).unwrap();
        let before = graph.node_aspect_version(nodes.source).unwrap();
        let route_before = graph.node_aspect_version(nodes.route).unwrap();
        let request = RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(commit.outcome().commit.commit_id.0),
        );
        let delivery = bridge
            .bind_signal_graph(&mut graph)
            .unwrap()
            .deliver_installed_correspondence(&installed, request);
        let TransitionOutcome::Success(receipt) = delivery else {
            panic!("installed Bridge delivery: {delivery:?}")
        };
        assert_eq!(receipt.source_load_attempts(), 1);
        assert_eq!(receipt.source_envelopes_loaded(), 1);
        assert_eq!(receipt.truth_targets_admitted(), expected_seeds);
        assert_eq!(receipt.signal_seeds_emitted(), expected_seeds);
        let after = graph.node_aspect_version(nodes.source).unwrap();
        assert_eq!(
            after != before,
            expected_seeds == 1,
            "only matching delivery changes the installed source"
        );
        assert_eq!(
            graph.node_aspect_version(nodes.route).unwrap(),
            route_before,
            "delivery targets the source, not the derived route"
        );
        oracle.change(name, amount);
        drop(selected_lease);
    }
    drop(source);
    let court = CompositeSupplyChainCourt::from_installed_graph(
        records,
        graph,
        nodes,
        bridge,
        installed,
        budgets::court(),
    );
    let root = court.bootstrap();
    let mut context = court.context(&root);
    assert_eq!(context.records.get().unwrap(), &oracle.records);
    let token = RuntimeWorldCancellationSource::new().token();
    let port = court.world.publication_port();
    let prepared = port
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let mut routed = None;
    let outcome = port.execute_with_signal(prepared, &mut context, &token, |tx| {
        // No manual invalidation: the installed graph carries Bridge's change.
        routed = Some(
            tx.read(nodes.route, &|view| nodes.evaluate(view))?
                .get(ROUTED),
        );
        Ok(())
    });
    let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
        panic!("sealed delivered graph publishes")
    };
    assert_eq!(routed, Some(oracle.route()));
    drop((done.consume(), root));
    court.finish();
}
