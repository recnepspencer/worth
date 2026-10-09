//! Spare capacity and the nested boxed payload are part of memory custody.
use super::*;
use worth_execution::ChargedBytes;
#[test]
fn boxed_authorization_and_spare_string_capacity_are_charged() {
    let mut query = String::with_capacity(64);
    query.push('q');
    let mut subject = String::with_capacity(128);
    subject.push('s');
    let mut authorization_subject = String::with_capacity(256);
    authorization_subject.push('a');
    let authorization = WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        authorization_subject,
    );
    let denial = WorthQueryApplicationOneShotDenial {
        kind: WorthQueryApplicationOneShotDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        ),
        payload: std::sync::Arc::new(OneShotDenialPayload {
            authorization_denial: Some(Box::new(authorization)),
            query,
            subject,
            custody: std::sync::OnceLock::new(),
        }),
    };
    // One shared Arc payload, two reference counters, and the original owned allocations.
    let expected = std::mem::size_of::<OneShotDenialPayload>()
        + 2 * std::mem::size_of::<usize>()
        + 64
        + 128
        + 256
        + std::mem::size_of::<WorthQueryOperationAuthorizationDenial>()
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
    use super::super::super::WorthQueryManagedDerivedViewDenial as ViewDenial;
    use worth_execution::{ExecutionMap, MapKernelFailure, MapOutcome, MapPartition, MapStop};
    let bytes = (std::mem::size_of::<OneShotDenialPayload>()
        + 2 * std::mem::size_of::<usize>()
        + "query".len()
        + "subject".len()) as u64;
    let budget = worth_execution::SerialMemoryBudget::new(1 << 20);
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
    let (result, _) = worth_execution::ExecutionWorkCeiling::new(u64::MAX)
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
        budget.reserve(budget.limit()).unwrap_err().admitted,
        budget.limit() - bytes
    );
    drop(error);
    assert_eq!(
        budget.reserve(budget.limit()).unwrap_err().admitted,
        budget.limit() - bytes
    );
    drop(clone);
    assert!(budget.reserve(budget.limit()).is_ok());
}
