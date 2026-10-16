//! Original root output roles remain inspectable without a producer receipt.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandSettlement,
    WorthQueryApplicationProgramOutputSettlement, WorthQueryOutputSettlementPosture,
    WorthQueryProgramOutputOperationAdvance, WorthQuerySettledProgramOutputOperation,
};

#[test]
fn no_receipt_root_roles_remain_borrowed_from_original_complete_tree() {
    let _guard = checkpoint_recovery_test_guard();
    let mut application = install(None);
    let settled = {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        // A genuine producer receipt supplies independent original role evidence.
        let initial = settle_tree(&application, &request);
        let original = initial
            .settlement()
            .root_receipt()
            .unwrap()
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id();
        initial
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));

        change_suffix_preserving_root_input(&application, &request);

        let settled = settle_tree(&application, &request);
        {
            let aggregate: &WorthQueryApplicationProgramOutputSettlement<PlanarQuery> =
                settled.settlement();
            let root: &WorthQueryApplicationOutputDemandSettlement<PlanarQuery> =
                aggregate.root_settlement();
            assert!(
                aggregate.root_receipt().is_none(),
                "a real Stable root has no producer receipt"
            );
            assert!(root.application_commit_receipt().is_none());
            assert_eq!(
                root.posture(),
                WorthQueryOutputSettlementPosture::StableReused
            );
            assert_eq!(
                root.producer_contacts_in_this_demand(),
                0,
                "the root producer was not replayed"
            );
            assert_eq!(
                root.outputs_of::<PlanarOutputs>()
                    .unwrap()
                    .entity::<PlanarAnchorOutput<CheckpointSchema>>()
                    .unwrap()
                    .entity_id(),
                original
            );
        }
        // The root borrow supplies no complete-tree currentness. Use the
        // original owning parent to validate all members and read at its basis.
        assert_eq!(settled.settlement().output_count(), 2);
        let read = settled
            .read_current(
                &request,
                PlanarOutputRead {
                    body_key: "anchor-a".into(),
                },
            )
            .unwrap();
        assert_eq!(read.rows()[0].value, length(2));
        drop(read);
        settled
    };
    assert_eq!(
        application.close_conditional_runtime().unwrap_err().kind(),
        primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained,
        "inspecting root roles leaves original parent custody retained"
    );
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        settled
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    }
    application
        .close_conditional_runtime()
        .expect("original tree and runtime close lawfully");
}

fn change_suffix_preserving_root_input(
    application: &support::Application,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
) {
    // Deliberate native court setup: use CheckpointProgram's actual declared
    // PlanarEdit action to change a suffix outside the root producer's input.
    // The schema's specialized PlanarSourceAdjustment binding is not a program
    // action. Adjust writes the same PositionY fact and starts no output demand.
    let input = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let original_input = input.rows().to_vec();
    drop(input);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    let edited = request
        .advanced()
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-b".into(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: "anchor-b".into(),
                    replacement_y: length(2),
                },
            ]),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6619_u64)
        .execute_in_program::<CheckpointProgram>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        edited,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    drop(source);
    let edited = request
        .query(PlanarRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        edited.rows()[0].y,
        length(2),
        "the source suffix genuinely changed"
    );
    drop(edited);
    let input = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        input.rows(),
        original_input.as_slice(),
        "declared root input is unchanged"
    );
    drop(input);
}

fn settle_tree(
    application: &support::Application,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
) -> WorthQuerySettledProgramOutputOperation<CheckpointSchema, CheckpointProgram, CheckpointRoot> {
    let mut operation = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .retain_program_outputs::<CheckpointProgram, CheckpointRoot>(application)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    // Test hang guard for a finite installed tree; never retry a refusal.
    for _ in 0..256 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => return settled,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    panic!("the original finite read tree must settle")
}
