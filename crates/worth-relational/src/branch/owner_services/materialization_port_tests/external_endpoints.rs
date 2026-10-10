use super::*;

#[test]
fn external_endpoint_group_restores_only_its_exact_relation_endpoints() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let external = crate::tests::support::create_entity(&runtime, "external-endpoint");
    let created = create_group_with_external_endpoint(&runtime, external);
    let internal = changed_entities(&created)[0];
    let relation = changed_relations(&created)[0];
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the output-owned boundary link has a current basis");
    let suspended = services
        .materialization_port()
        .suspend_generated_materialization(&basis, &[internal])
        .expect("a generated relation may retain an endpoint outside its output group");
    assert!(suspended
        .custody
        .records()
        .contains(&RelationalMaterializationRecord::Relation {
            relation_id: relation,
            kind_id: KindId(2),
            source: external,
            target: internal,
        }));
    assert!(runtime
        .read_truth()
        .read_snapshot(&suspended.commit.snapshot, execution)
        .is_none());

    let (_, unavailable_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the suspended boundary link retains an owner basis");
    let port = services.materialization_port();
    assert_eq!(
        port.retained_entity_kind(&unavailable_basis, &suspended.custody, external)
            .expect("the exact live retained endpoint remains observable without payload access"),
        KindId(1)
    );
    assert!(
        matches!(
            port.retained_entity_kind(&unavailable_basis, &suspended.custody, internal),
            Err(RelationalMaterializationError::CandidateManifestMismatch)
        ),
        "custody-owned generated identity cannot be reclassified as retained"
    );
    assert!(
        matches!(
            port.retained_entity_kind(&basis, &suspended.custody, external),
            Err(RelationalMaterializationError::SuspensionCommitMismatch)
        ),
        "the old complete basis cannot stand in for the exact suspension"
    );
    let foreign = runtime_with_test_schema();
    assert!(
        matches!(
            foreign
                .owner_component_services()
                .materialization_port()
                .retained_entity_kind(&unavailable_basis, &suspended.custody, external),
            Err(RelationalMaterializationError::BranchMismatch)
        ),
        "a foreign owner cannot observe through transported custody and basis"
    );
    let (_, foreign_basis) = foreign
        .owner_component_services()
        .basis_port()
        .observe_branch(&foreign.main_branch_identity())
        .expect("the foreign owner admits its own basis");
    assert!(
        matches!(
            foreign
                .owner_component_services()
                .materialization_port()
                .retained_entity_kind(&foreign_basis, &suspended.custody, external),
            Err(RelationalMaterializationError::BranchMismatch)
        ),
        "own admitted basis cannot make another branch's custody authoritative"
    );

    let failed = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            suspended.custody,
            vec![entity_materialization(
                internal,
                crate::tests::support::name_field_patch("internal-endpoint"),
            )],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: internal,
                target: external,
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            }],
        )
        .expect_err("endpoint reversal cannot satisfy exact custody");
    assert!(matches!(
        failed.error,
        RelationalMaterializationError::CandidateManifestMismatch
    ));

    let restored = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            failed.custody,
            vec![entity_materialization(
                internal,
                crate::tests::support::name_field_patch("internal-endpoint"),
            )],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: external,
                target: internal,
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            }],
        )
        .expect("the exact external endpoint relationship restores");
    let read = runtime
        .read_truth()
        .read_snapshot(&restored.snapshot, execution)
        .expect("exact restoration resumes ordinary reads");
    assert_eq!(read.relations()[0].source, external);
    assert_eq!(read.relations()[0].target, internal);
    assert!(runtime
        .read_truth()
        .read_snapshot(&created.snapshot, execution)
        .is_some());
}
