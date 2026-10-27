//! Publication gates on publication failures, using the actual proposed state.
use super::*;

#[test]
fn a_native_publication_failure_blocks_custom_preparation_while_a_commit_effect_at_that_point_continues(
) {
    for effect in [Effect::BlockPublication, Effect::BlockCommit] {
        let hooks = Arc::new(Hooks::default());
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(acyclicity_and_connectivity_registry())
            .invariant_catalog(InvariantCatalog {
                registrations: vec![InvariantRegistration::for_rule(
                    InvariantRule::MaxSnapshotEntities(0),
                    Point::SnapshotPublication,
                    effect,
                )],
            })
            .custom_invariant(
                CustomInvariantRegistration::new(AtPoint {
                    probe: ScopeProbe(Arc::clone(&hooks)),
                    point: Point::SnapshotPublication,
                    checked: true,
                })
                .unwrap(),
            )
            .build();
        // This is the genuine empty runtime; no fabricated entity identity is used.
        let fixture = DagFixture {
            runtime,
            nodes: Vec::new(),
        };
        let before = fixture.head();
        let batch = WorkerIntentBatch::new("proposed-one").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: ClientKey::raw("proposed-one"),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        ));
        let lease = super::super::super::fixture::request(
            8 * 1024 * 1024,
            4_000_000,
            CancellationToken::new(),
            None,
        );
        let outcome = fixture.validate(&ExecutionRequest::leased(&lease), batch);
        if effect == Effect::BlockPublication {
            let Err(TransactionCommitError::Publication { error, .. }) = outcome else {
                panic!("the original native publication denial is required");
            };
            assert!(
                matches!(
                    error
                        .invariant_violation()
                        .map(|violation| &violation.fields),
                    Some(InvariantViolationFields::SnapshotEntityLimit {
                        visible_entities: 1,
                        limit: 0,
                        ..
                    })
                ),
                "{error:?}"
            );
            assert_eq!(hooks.observed(), (0, 0));
        } else {
            let candidate = outcome.unwrap();
            let result = &candidate.snapshot_publication;
            assert_eq!(result.results().len(), 3);
            assert_eq!(
                result.results()[0].execution_point,
                Point::SnapshotPublication
            );
            assert_eq!(result.results()[0].failure_effect, Effect::BlockCommit);
            assert!(matches!(
                &result.results()[0].rule,
                InvariantReportedRule::Native(InvariantRule::MaxSnapshotEntities(0))
            ));
            assert!(
                matches!(&result.results()[0].verdict, InvariantVerdict::Violation(violation)
                if matches!(violation.fields, InvariantViolationFields::SnapshotEntityLimit { visible_entities: 1, limit: 0, .. }))
            );
            assert!(matches!(
                &result.results()[1].rule,
                InvariantReportedRule::Native(InvariantRule::ConnectivityMinimumContract(contract))
                    if contract.contract_id.as_str() == "reachable_anchor"
            ));
            assert!(matches!(
                result.results()[1].verdict,
                InvariantVerdict::Pass
            ));
            assert!(matches!(
                &result.results()[2].rule,
                InvariantReportedRule::Custom(_)
            ));
            assert_eq!(result.decision_log().len(), 3);
            assert_eq!(
                result.metadata().proof_boundary().unwrap().packet_count(),
                3
            );
            assert_eq!(hooks.observed(), (1, 1));
            drop(candidate);
        }
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
