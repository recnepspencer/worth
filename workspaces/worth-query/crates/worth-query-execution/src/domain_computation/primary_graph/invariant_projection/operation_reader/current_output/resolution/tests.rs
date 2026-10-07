use super::{classify_current_entities, CurrentOutputCardinality};

#[test]
fn current_role_cardinality_ignores_repeated_identity_but_not_distinct_outputs() {
    assert_eq!(
        classify_current_entities(Vec::<u64>::new()),
        CurrentOutputCardinality::Missing
    );
    assert_eq!(
        classify_current_entities([11, 11]),
        CurrentOutputCardinality::Unique(11)
    );
    assert_eq!(
        classify_current_entities([11, 11, 12, 12]),
        CurrentOutputCardinality::Ambiguous(vec![11, 12])
    );
}

#[test]
fn a_full_ledger_reaches_the_handler_and_projection_as_capacity_exhausted() {
    let (handler, projection) = crate::domain_computation::primary_graph::tests::fixture::retained_output_capacity::project_at_full_ledger();
    assert_eq!(
        handler,
        super::super::WorthQueryCurrentOutputDenialKind::RetentionCapacityExhausted
    );
    assert_eq!(projection.kind(), crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
        crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind::RetentionCapacityExhausted));
}

#[test]
fn index_loss_cannot_certify_own_write_output_at_the_retained_postcommit_observation() {
    use crate::domain_computation::primary_graph::{
        output_lineage::own_write_fixture::with_committed_own_write,
        tests::{
            application_attempt::{authenticated_principal, resolved_account},
            fixture::{
                live_scope,
                retained_output_capacity::{ReadRetainedAccount, RetainedFamily},
            },
        },
        WorthQueryCurrentOutputDenialKind,
    };
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    with_committed_own_write(|world, receipt, _, resources| {
        let request = live_scope();
        let principal = authenticated_principal(world, &request);
        let account = resolved_account(world, "2", &request);
        let graph = world.application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        handle
            .output_lineage
            .lock()
            .unwrap()
            .install_output_families(std::collections::BTreeMap::from([(
                "test.retained-account-capacity".to_owned(),
                vec![(
                    receipt.output_correspondence().binding_type().unwrap(),
                    "retained-account".to_owned(),
                )],
            )]));
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ReadRetainedAccount::reference())
            .unwrap();
        let admitted = world
            .selected_product()
            .authorize_operation(
                &principal,
                &account,
                &operation,
                TypedMutationPreconditions::new(),
                &request,
            )
            .unwrap();
        crate::domain_computation::primary_graph::output_lineage::own_write_fixture::evict_index_with_unrelated_write(world, &resources);
        let answer = world.invariant.project_admitted_operation(&admitted, |reader, root| {
            let positioned = reader.reader.runtime.read_truth().positioned_snapshot(reader.reader.snapshot).unwrap();
            let owner = &reader.reader.invalidation_owner;
            assert!(matches!(owner.currentness(&positioned, receipt.exact_output_settlement().unwrap(), &mut owner.edit_admission()).unwrap(),
                crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness::FullVerificationRequired(_)),
                "the unrelated publication must have removed the retained observation's index image");
            reader.current_output::<RetainedFamily, _>(root).map(|_| ())
                .map_err(|denial| denial.kind())
        }).unwrap().into_parts().0;
        assert_eq!(
            answer,
            Err(WorthQueryCurrentOutputDenialKind::StaleSource),
            "rebased x=2 and an unchanged witness cannot certify output computed from x=1"
        );
    });
}
