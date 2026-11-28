//! Body producer coupling follows Stage.FeatureBody -> BodyKey through real Query.
use super::super::{correction_courts::install_program, lifecycle::*, program::*};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
fn lifecycle<S: Scenario>() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = request
        .advanced()
        .mutate(Create {
            create_co: true,
            co_anchors: vec![20],
            scope: 1,
            id: 10,
            depth: 1,
        })
        .without_source()
        .idempotency(&"body-and-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            created,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{created:?}"
    );
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = created
    else {
        unreachable!()
    };
    let old_stage = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<super::super::co_roles::CoCreated<S, 0>>()
        .unwrap()
        .expect("created stage")
        .entity_id();
    let settle_body = || {
        let mut pending = request
            .demand(Demand(10))
            .retain_program_outputs::<Program, Graph<S>>(&host)
            .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
        for attempt in 0..32 {
            match pending.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
                WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                    settled
                        .close(&request)
                        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    panic!("{denial:?}")
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    panic!("{:?}", terminal.denial())
                }
            }
            assert!(attempt < 31, "initial body did not settle");
        }
    };
    settle_body();
    producer::handler::reset_contacts();
    let retired = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 20,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"retire-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    if S::MODE == 10 {
        require_missing_owner(
            retired,
            created
                .outputs_of::<CreatedOutputs>()
                .unwrap()
                .entity::<Created<S>>()
                .unwrap()
                .entity_id(),
        );
        assert_eq!(producer::handler::contacts(), 0);
        for id in [20, 101, 102] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
        assert!(request.query(query::Input { id: 103 }).execute().is_err());
        return;
    }
    assert!(
        matches!(
            retired,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{retired:?}"
    );
    assert_eq!(
        producer::handler::contacts(),
        1,
        "one actual body producer contact"
    );
    assert!(
        request.query(query::Input { id: 20 }).execute().is_err(),
        "stage remains live"
    );
    assert!(
        request.query(query::Input { id: 102 }).execute().is_err(),
        "old body cell remains live"
    );
    assert!(
        request.query(query::Input { id: 101 }).execute().is_ok(),
        "body carrier survives"
    );
    assert!(
        request.query(query::Input { id: 103 }).execute().is_ok(),
        "body successor cell is in the same committed state"
    );
    settle_body();
    assert_eq!(
        producer::handler::contacts(),
        1,
        "demanding the coupled successor must reuse its own settled output"
    );

}

fn require_missing_owner<D, R>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
    source: worth_relational::facade::identity::EntityId,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("missing owner must refuse")
    };
    let failure = denial
        .authored_retirement_error()
        .expect("typed Native coupling refusal");
    assert!(matches!(failure.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::MissingCoupledOwner)), "{failure:?}");
    assert_eq!(
        failure.context.affected_records,
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
}
#[test]
fn retiring_a_stage_runs_its_bodys_preserve_in_the_same_publication() {
    lifecycle::<Model<6>>();
}
#[test]
fn retiring_a_stage_without_declaring_its_required_body_refuses_before_effect() {
    lifecycle::<Model<10>>();
}
#[test]
fn a_coupled_body_keeps_its_native_aggregate_membership_consequences() {
    lifecycle::<Model<14>>();
}
