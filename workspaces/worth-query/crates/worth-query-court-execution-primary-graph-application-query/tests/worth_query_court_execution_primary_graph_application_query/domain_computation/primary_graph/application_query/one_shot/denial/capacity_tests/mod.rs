//! Spare capacity and the nested boxed payload are part of memory custody.
use worth_execution::ChargedBytes;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::one_shot::denial::authorization_denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::one_shot::denial::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::one_shot::denial::retain_reconstruction_result;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::one_shot::denial::OneShotDenialPayload;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
#[test]
fn boxed_authorization_and_spare_string_capacity_are_charged() {
    let mut query = String::with_capacity(64);
    query.push('q');
    let mut subject = String::with_capacity(128);
    subject.push('s');
    let mut authorization_subject = String::with_capacity(256);
    authorization_subject.push('a');
    let authorization = WorthQueryOperationAuthorizationDenial::new_for_test(
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied),
        authorization_subject,
    );
    let denial = WorthQueryApplicationOneShotDenial::new_for_test(WorthQueryApplicationOneShotDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::PermissionDenied,
            ),
        ), std::sync::Arc::new(OneShotDenialPayload::new_for_test(Some(Box::new(authorization)), query, subject, std::sync::OnceLock::new())));
    // One shared Arc payload, two reference counters, and the original owned allocations.
    let expected = std::mem::size_of::<OneShotDenialPayload>()
        + 2 * std::mem::size_of::<usize>()
        + 64
        + 128
        + 256
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenial>()
        // The primary kind box is separate from the one-element causes vector.
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenialKind>()
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenialKind>();
    assert_eq!(
        denial.additional_charged_bytes(),
        expected as u64,
        "the box allocation, nested cause vector and all capacities must be reserved"
    );
}

#[test]
fn denial_clones_share_payload_and_keep_its_reservation_until_last_drop() {
    let denial = denial(
        WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded,
        "query",
        "subject",
    );
    use worth_execution::ExecutionMap;
    use worth_execution::MapKernelFailure;
    use worth_execution::MapOutcome;
    use worth_execution::MapPartition;
    use worth_execution::MapStop;
    use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewDenial as ViewDenial;
    let bytes = (std::mem::size_of::<OneShotDenialPayload>()
        + 2 * std::mem::size_of::<usize>()
        + "query".len()
        + "subject".len()) as u64;
    let budget = worth_execution::SerialMemoryBudget::new(Some(1 << 20));
    let limit = budget
        .limit()
        .expect("the custody proof declares a finite limit");
    let request = worth_execution::SerialRequest::from_memory(
        budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let root = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let (result, _) = worth_execution::ExecutionWorkCeiling::new(None)
        .run_serial(&request, || {
            let identity = worth_foundational::PartitionIdentity::new(1);
            let map = ExecutionMap::<_, u64>::try_from_declared_partitions(
                vec![identity],
                vec![MapPartition {
                    identity,
                    value: denial,
                    read_keys: vec![],
                    write_keys: vec![],
                    kernel_scratch_bytes: 0,
                    max_result_bytes: bytes,
                }],
            )
            .unwrap();
            let outcome = map.run_owned(
                None,
                |denial, _| -> Result<u64, MapKernelFailure<ViewDenial>> {
                    Err(MapKernelFailure::Domain(ViewDenial::ReadDenied {
                        root,
                        denial,
                    }))
                },
            );
            let MapOutcome::Stopped {
                reason:
                    MapStop::Failure {
                        cause: MapKernelFailure::Domain(error),
                        ..
                    },
                ..
            } = outcome
            else {
                panic!("the worker must return its original read denial")
            };
            // The allowance is gone at the map boundary; only the returned payload is funded.
            retain_reconstruction_result::<()>(None, Err(error), |_| unreachable!())
        })
        .unwrap();
    let error = result.unwrap_err();
    let clone = error.clone();
    assert_eq!(
        budget.reserve(limit).unwrap_err(),
        worth_execution::MemoryLimitDenial::Capacity {
            requested: limit,
            admitted: limit - bytes,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    drop(error);
    assert_eq!(
        budget.reserve(limit).unwrap_err(),
        worth_execution::MemoryLimitDenial::Capacity {
            requested: limit,
            admitted: limit - bytes,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    drop(clone);
    assert!(budget.reserve(limit).is_ok());
}
