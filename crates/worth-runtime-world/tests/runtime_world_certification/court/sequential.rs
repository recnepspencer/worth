use super::oracle::CompositeWorldOracle;
use super::routing::{RoutingContext, INPUT, ROUTED};
use super::*;

#[test]
fn bootstrap_and_real_component_publications_match_independent_cargo_oracle() {
    let mut court = CompositeSupplyChainCourt::compile();
    let mut expected = CompositeWorldOracle::bootstrap();
    let signal_port = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let signal_before = signal_port.owner_service_cost_snapshot().unwrap();
    let relational_before = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let root = court.bootstrap();
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        relational_before,
        "bootstrap: zero Relational movement"
    );
    assert_eq!(
        signal_port
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        signal_before.canonical_movements(),
        "bootstrap: zero Signal movement"
    );
    assert_eq!(
        court.records.read(root.basis().relational_basis()),
        expected.records,
        "oracle: root records"
    );
    assert_eq!(court.installed.target_count(), 1);
    court
        .bridge
        .runtime_world_correspondence_port()
        .admit_installed_basis(&court.installed)
        .unwrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let port = court.world.publication_port();
    let candidate = court
        .records
        .candidate(root.basis().relational_basis(), "grain", "5");
    let intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(candidate);
    let prepared = port
        .prepare_without_signal(root.clone(), intent, &token, None)
        .expect("World prepare cargo change");
    let RuntimeWorldPublicationOutcome::Performed(done) =
        port.execute_without_signal(prepared, &token)
    else {
        panic!("World cargo change must perform")
    };
    let receipt = done.consume();
    assert_eq!(
        signal_port
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        signal_before.canonical_movements(),
        "Relational-only: zero Signal movement"
    );
    assert_ne!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        relational_before,
        "direct Relational movement"
    );
    expected.change("grain", "5");
    let updated = court.observe(&root);
    assert_eq!(
        court.records.read(updated.basis().relational_basis()),
        expected.records,
        "oracle: exact successor cargo"
    );
    assert_eq!(
        updated.basis().signal_basis().descriptor(),
        root.basis().signal_basis().descriptor(),
        "retained sibling basis"
    );
    assert!(receipt
        .component_results()
        .relational_commit_result()
        .is_some());
    let prepared = port
        .prepare_with_signal(
            updated.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let mut context = RoutingContext {
        records: std::sync::Arc::new(std::sync::OnceLock::from(
            court.records.read(updated.basis().relational_basis()),
        )),
    };
    let nodes = court.nodes;
    let mut routed = None;
    let outcome = port.execute_with_signal(prepared, &mut context, &token, |tx| {
        tx.mark_changed(nodes.source, INPUT)?;
        let routed_version = tx.read(nodes.route, &|view| nodes.evaluate(view))?;
        routed = Some(routed_version.get(ROUTED));
        Ok(())
    });
    let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
        panic!("World routing must perform")
    };
    let signal_receipt = done.consume();
    assert_eq!(
        signal_port
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        signal_before.canonical_movements() + 1,
        "direct Signal movement"
    );
    let reference = signal_port
        .issue_managed_branch_reference(updated.basis().signal_basis())
        .unwrap();
    let direct_signal = signal_port.observe_current(&reference).unwrap();
    assert_eq!(
        routed,
        Some(expected.route()),
        "oracle: real evaluated routed tonnes"
    );
    let final_head = court.observe(&updated);
    assert_eq!(
        court.records.read(final_head.basis().relational_basis()),
        expected.records
    );
    assert_ne!(
        final_head.basis().signal_basis().descriptor(),
        updated.basis().signal_basis().descriptor()
    );
    assert_eq!(
        final_head.basis().signal_basis().descriptor(),
        direct_signal.descriptor(),
        "product names directly observed Signal successor"
    );
    assert_eq!(
        court
            .world
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        expected.occurrences + 1
    );
    drop((
        root,
        updated,
        final_head,
        receipt,
        signal_receipt,
        direct_signal,
        reference,
    ));
    court.finish();
}

#[test]
fn combined_publication_routes_actual_settled_relational_successor() {
    for (amount, include_steel) in [("5", true), ("9", true), ("9", false)] {
        let mut court = CompositeSupplyChainCourt::compile_with_steel(include_steel);
        let signal_port = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let signal_before = signal_port
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements();
        let root = court.bootstrap();
        let mut oracle = CompositeWorldOracle::bootstrap();
        if !include_steel {
            oracle
                .records
                .links
                .remove(&("manifest".into(), "steel".into()));
        }
        let candidate = court
            .records
            .candidate(root.basis().relational_basis(), "grain", amount);
        let token = RuntimeWorldCancellationSource::new().token();
        let port = court.world.publication_port();
        let intent =
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate);
        let prepared = port
            .prepare_with_signal(root.clone(), intent, &token, None)
            .unwrap();
        // The callback observes the actual settled owner successor, then freezes
        // that exact read for this Signal transaction; it never predicts a result.
        let input = std::sync::Arc::new(std::sync::OnceLock::new());
        let mut context = RoutingContext {
            records: input.clone(),
        };
        let mut observed_route = None;
        let mut relational_input = None;
        let nodes = court.nodes;
        let outcome = port.execute_with_signal(prepared, &mut context, &token, |tx| {
            let basis = court
                .records
                .runtime
                .observe_branch(&court.records.runtime.main_branch_identity())
                .unwrap()
                .1;
            input.set(court.records.read(&basis)).unwrap();
            relational_input = Some(basis.admission_identity().clone());
            tx.mark_changed(nodes.source, INPUT)?;
            let versions = tx.read_many(&[nodes.route], &|view| nodes.evaluate(view))?;
            observed_route = Some(versions[0].get(ROUTED));
            Ok(())
        });
        let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
            panic!("combined owner progression must perform")
        };
        let receipt = done.consume();
        oracle.change("grain", amount);
        assert_eq!(
            input.get().unwrap(),
            &oracle.records,
            "oracle: settled input to real Signal evaluation"
        );
        assert_eq!(
            observed_route,
            Some(oracle.route()),
            "oracle: within capacity versus overloaded voyage"
        );
        let head = court.observe(&root);
        assert_eq!(
            relational_input,
            Some(head.basis().relational_basis().admission_identity().clone()),
            "callback reads the exact performed Relational successor"
        );
        assert_eq!(
            signal_port
                .owner_service_cost_snapshot()
                .unwrap()
                .canonical_movements(),
            signal_before + 1
        );
        let reference = signal_port
            .issue_managed_branch_reference(root.basis().signal_basis())
            .unwrap();
        let direct = signal_port.observe_current(&reference).unwrap();
        assert_eq!(
            direct.descriptor(),
            head.basis().signal_basis().descriptor(),
            "combined World binds directly observed Signal successor"
        );
        assert_eq!(
            court.records.read(head.basis().relational_basis()),
            oracle.records
        );
        drop((root, head, receipt, reference, direct));
        court.finish();
    }
}
