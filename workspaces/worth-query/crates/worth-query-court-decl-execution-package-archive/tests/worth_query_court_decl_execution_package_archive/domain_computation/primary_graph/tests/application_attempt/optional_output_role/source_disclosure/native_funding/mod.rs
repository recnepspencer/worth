//! Private capture funding evidence; these payloads do not establish source acceptance.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::settlement::observation::court::CourtOriginalSourceInspectionDisclosureForTest;
use crate::domain_computation::primary_graph::tests::application_attempt::optional_output_role::source_disclosure::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::producer_observation::provenance::ProvenanceOriginalSourceInspectionDisclosureForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_disclosure::inspection_capture::capture_inspection;
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;

#[test]
fn native_serial_capture_uses_effective_ancestor_funding_and_releases_its_last_payload() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let original = OptionalOutputInput {
        status: "open".to_owned(),
        plan: OptionalOutputPlan::RequiredOnly,
    };
    let (account, input) = world.application.with_application_advancement(&request, |phase| {
        let account = resolved_account(&phase.test_host_request_for_test(), &world, "open", &request);
        let input = world.application.capture_operation_input_in_advancement::<<OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding>(
            &phase, &program_publication_access(), &request, &original,
        ).unwrap();
        (account, input)
    }).unwrap();
    let native = SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&native);
    assert!(!execution.has_accounting());
    let (payload, _) = execution
        .run_native_with_checkpoint(|context| {
            assert!(
                !context.has_accounting(),
                "the real native serial scope has no memory owner"
            );
            let scope = account.retain_scope_selector_funded_for_test(&execution).unwrap();
            capture_inspection::<IdentityExecutionSchema, OptionalOutputMutationBinding>(
                &execution,
                &world.application,
                input.input(),
                Some(&scope),
            )
            .unwrap()
            .unwrap()
        })
        .unwrap();
    let (foreign, _) = execution
        .run_native_with_checkpoint(|_| {
            let scope = account.retain_scope_selector_funded_for_test(&execution).unwrap();
            capture_inspection::<IdentityExecutionSchema, OptionalOutputMutationBinding>(
                &execution,
                &world.application,
                input.input(),
                Some(&scope),
            )
            .unwrap()
            .unwrap()
        })
        .unwrap();
    payload
        .clone()
        .court_public_settlement_rejects_foreign_receipt(foreign.clone());
    payload
        .clone()
        .court_rejects_present_foreign_disclosure(foreign);
    assert_eq!(payload.scope_for_test().subject_for_test(), account.entity_id_for_test());
    payload.clone().court_public_settlement_rejects_disclosure();
    drop(payload);

    for saturate in [false, true] {
        let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
        let parent =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let mut retained = None;
        let mut refused_capture = false;
        let handoff = ExecutionRequest::serial(&parent).run_native_with_checkpoint(|context| {
            assert!(context.has_accounting());
            assert!(!execution.has_accounting(), "the original serial backing itself still has no accounting");
            let scope = account.retain_scope_selector_funded_for_test(&execution).unwrap();
            let obstruction = saturate.then(|| budget.reserve(budget.limit().unwrap() - budget.charged_memory_bytes() - 1).unwrap());
            let captured = capture_inspection::<IdentityExecutionSchema, OptionalOutputMutationBinding>(
                &execution, &world.application, input.input(), Some(&scope),
            );
            if saturate {
                let Err(denial) = captured else { panic!("the actual inherited memory owner must refuse capture") };
                assert!(matches!(denial.kind(), WorthQueryApplicationAttemptDenialKind::ExecutionRequest(
                    worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit { .. }, ..
                    }
                )));
                refused_capture = true;
                drop(obstruction);
            } else {
                retained = captured.unwrap();
            }
        });
        if saturate {
            assert!(refused_capture);
            assert!(
                handoff.is_err(),
                "the original ancestor preserves its refusal at handoff"
            );
            assert!(retained.is_none());
            assert_eq!(
                budget.charged_memory_bytes(),
                0,
                "failed capture releases every inherited reservation"
            );
        } else {
            handoff.unwrap();
            let payload = retained.unwrap();
            let sibling = payload.clone();
            let held = budget.charged_memory_bytes();
            assert!(
                held > 0,
                "the actual inherited budget funds the payload beyond both request scopes"
            );
            drop(payload);
            assert_eq!(
                budget.charged_memory_bytes(),
                held,
                "one remaining shared payload retains the sole grant"
            );
            drop(sibling);
            assert_eq!(
                budget.charged_memory_bytes(),
                0,
                "the final capture owner releases exact inherited backing"
            );
        }
    }
}
