//! Denials from committed and applied-candidate aggregate pins.
use super::fixture::{commit, scope, start};
use super::schema;
use std::{cell::Cell, ops::ControlFlow};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    aggregate_identity::AggregateIdentity,
    identity::EntityId,
    mvcc::{
        RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
        RelationalCoupledAssemblyFailure, RelationalTransactionIntent,
    },
    runtime::{
        PinnedAggregateLineageScope, PinnedAggregateReadDenialKind as Denial, RelationalRuntimeApi,
    },
    schema::RelationalSchemaRegistry,
    transactions::{DeleteEntityIntent, EntityMutationIntent, MutationIntent},
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[test]
fn committed_named_pin_distinguishes_unavailable_members_and_wrong_kind() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema::registry(true, true))
        .build()
        .unwrap();
    let born = start(request, &runtime, true, true);
    let pin = runtime
        .read_truth()
        .pin_read_capability(&born.commit.snapshot, &[], request)
        .unwrap();
    let stale = EntityId::new(
        born.face.partition_id,
        born.face.local_slot_value(),
        born.face.generation_value() + 1,
    );
    for (member, expected) in [
        (stale, Denial::MemberUnavailable),
        (born.vertex, Denial::WrongMemberKind),
    ] {
        assert_eq!(
            pin.read_projected_aggregate(request, scope(member, 0))
                .unwrap_err()
                .kind,
            expected
        );
    }
    let deleted = commit(
        request,
        &runtime,
        super::disconnect(born.coupling)
            .push(super::delete_relation(born.incidence))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: born.face,
                },
            ))),
        Vec::new(),
    );
    let after = runtime
        .read_truth()
        .pin_read_capability(&deleted.snapshot, &[], request)
        .unwrap();
    assert_eq!(
        after
            .read_projected_aggregate(request, scope(born.face, 0))
            .unwrap_err()
            .kind,
        Denial::MemberUnavailable
    );
    assert_eq!(
        pin.read_projected_aggregate(request, scope(born.face, 0))
            .unwrap()
            .len(),
        2
    );
}

struct CandidateRead {
    face: EntityId,
    identity: AggregateIdentity,
    called: Cell<bool>,
}

impl RelationalAuthoredCoupledProducerAssembly for CandidateRead {
    fn assemble(
        &self,
        request: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let pin = candidate.pin_source(request)?;
        assert_eq!(
            pin.read_projected_aggregate(request, scope(self.face, 0))
                .unwrap_err()
                .kind,
            Denial::PublicationRequired
        );
        assert_eq!(
            pin.read_aggregate_lineage(
                request,
                PinnedAggregateLineageScope::for_definition(
                    schema::descriptor(true).definition,
                    self.identity,
                ),
            )
            .unwrap_err()
            .kind,
            Denial::PublicationRequired
        );
        self.called.set(true);
        Ok(ControlFlow::Continue(()))
    }
}

#[test]
fn named_aggregate_candidate_refusal_leaves_the_commit_publishable() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema::registry(true, true))
        .build()
        .unwrap();
    let (_, installed) = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap()
        .issue_application_publication_owner();
    let born = start(request, &runtime, true, true);
    let identity = super::read(request, &runtime, &born.commit.snapshot, born.face, 0).identity();
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let owners = runtime
        .readmit_edit_coupled_owners(request, &installed, &basis, born.face)
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        super::disconnect(born.coupling),
        request.allocation_policy(),
    )
    .unwrap();
    let tx = tx
        .bind_authored_coupled_producers(&installed, owners, None)
        .unwrap();
    let assembly = CandidateRead {
        face: born.face,
        identity,
        called: Cell::new(false),
    };
    let published = runtime
        .preparation_port()
        .prepare_authored_coupled_transaction(request, tx, &assembly)
        .unwrap()
        .unwrap()
        .commit(request, &runtime)
        .unwrap();
    assert!(assembly.called.get());
    assert_eq!(
        super::read(request, &runtime, &published.snapshot, born.face, 0).len(),
        1
    );
}
