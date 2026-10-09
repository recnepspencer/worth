//! Real prepared native records, pre-effect refusal and shared physical custody.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, Account, AccountIdentity, AccountLabel,
    AccountStatus,
};
use std::{alloc::Layout, collections::BTreeMap, num::NonZeroUsize};
use worth_execution::{
    CancellationToken, ExecutionAllocationDenialKind, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseDenial, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::{
    mvcc::{CompanionPreflightBudget, RelationalTransactionIntent},
    transactions::{AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn admission() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 1_000,
        maximum_preparation_bytes: 1_024,
    })
}

#[test]
fn native_prepared_records_are_admitted_before_effects_and_shared_until_last_drop() {
    let world = installed_authorization_world(true);
    let head = world.selected_product().product().selected_commit().clone();
    let graph = world.application.primary_provider.graph.clone();
    let kind = graph
        .layout
        .entity_kind(Account::reference().name())
        .unwrap();
    let locator = |entity, aspect, field| {
        graph
            .layout
            .field_locator(entity, aspect, field)
            .unwrap()
            .clone()
    };
    let identity = AccountIdentity::reference();
    let status = AccountStatus::reference();
    let label = AccountLabel::reference();
    let value = StringApplicationValueBinding::encode(&"prepared-custody".to_owned()).unwrap();
    let fields = BTreeMap::from([
        (
            locator(identity.entity(), identity.aspect(), identity.field()),
            value.clone(),
        ),
        (
            locator(status.entity(), status.aspect(), status.field()),
            value.clone(),
        ),
        (
            locator(label.entity(), label.aspect(), label.field()),
            value,
        ),
    ]);
    let batch = WorkerIntentBatch::new("prepared-touched-custody").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw("prepared-custody"),
            fields: AspectFieldPatch::from(fields),
        }),
    ));
    let (_, product, _basis) = world.selected_product().into_parts();
    let candidate = graph.with_runtime_mut(|runtime| {
        let mut transaction = runtime
            .begin_branch_transaction(
                product.relational_basis(),
                RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        runtime
            .prepare_branch_transaction(transaction, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let exact = candidate
        .with_prepared_changed_records(|records| records.to_vec())
        .unwrap();
    assert_eq!(
        exact.len(),
        1,
        "one actual entity create has one changed record"
    );
    let quote = u64::try_from(
        Layout::array::<WorthQueryTouchedRecordIdentity>(exact.len())
            .unwrap()
            .size(),
    )
    .unwrap();
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap();
    let zero = authority.request_lease(request(0)).unwrap();
    let denied = match PreparedTouchedRecords::prepare(
        &candidate,
        &mut admission(),
        ExecutionAllocationPolicy::Execution(&zero),
    ) {
        Err(WorthQueryProviderSessionCommitStop::PreEffectDenied(failure)) => failure,
        _ => panic!("exact pre-effect allocation refusal is required"),
    };
    let cause = denied.allocation_denial().unwrap();
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::ResourceExhausted)
    );
    assert_eq!(cause.requested_payload_bytes(), Some(quote));
    assert_eq!(world.selected_product().product().selected_commit(), &head);
    assert_eq!(
        candidate.with_prepared_changed_records(|records| records.to_vec()),
        Some(exact.clone())
    );
    drop(zero);
    let stopped_request = request(quote);
    let stop = stopped_request.cancellation.clone();
    let stopped = authority.request_lease(stopped_request).unwrap();
    stop.cancel();
    let cancelled = match PreparedTouchedRecords::prepare(
        &candidate,
        &mut admission(),
        ExecutionAllocationPolicy::Execution(&stopped),
    ) {
        Err(WorthQueryProviderSessionCommitStop::PreEffectDenied(failure)) => failure,
        _ => panic!("cancelled backing cannot reach publication"),
    };
    assert_eq!(
        cancelled.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    assert_eq!(world.selected_product().product().selected_commit(), &head);
    drop(stopped);

    let parent = authority.request_lease(request(quote)).unwrap();
    let child = parent.child(request(quote)).unwrap();
    let prepared = PreparedTouchedRecords::prepare(
        &candidate,
        &mut admission(),
        ExecutionAllocationPolicy::Execution(&child),
    )
    .unwrap_or_else(|_| panic!("exact layout fits"));
    let retained = Arc::clone(&prepared.records);
    assert_eq!(retained.charged_payload_bytes(), Some(quote));
    drop(child);
    assert!(matches!(
        parent.reserve_memory(1),
        Err(LeaseDenial::ResourceExhausted)
    ));
    let publication = product
        .publication_binding()
        .prepare_relational_candidate(candidate, &live_scope(), false)
        .expect("the actual product owner admits the prepared native candidate");
    let outcome = publication.execute();
    let worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) = outcome
    else {
        panic!("the prepared native candidate must publish through its product owner: {outcome:?}");
    };
    let committed = performed
        .component_results()
        .relational_commit_result()
        .expect("the performed product publication includes the native commit");
    let published = prepared.verify_performed(&committed.changed_records);
    drop(performed.consume());
    assert!(Arc::ptr_eq(&published, &retained));
    assert_eq!(
        published
            .iter()
            .map(|record| record.record().clone())
            .collect::<Vec<_>>(),
        exact
    );
    drop(published);
    assert!(matches!(
        parent.reserve_memory(1),
        Err(LeaseDenial::ResourceExhausted)
    ));
    drop(retained);
    assert_eq!(parent.reserve_memory(quote).unwrap().charged_bytes(), quote);
}
