//! Corruption changes one actual Query writer; Native recomputes its true owner.
use super::super::{
    correction_courts::{install_definition, try_reverse},
    lifecycle::*,
    *,
};
use super::{
    multi::{create, settle, TwoProgram},
    producer::handler,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};
#[test]
fn stale_and_birth_participants_refuse_with_the_actual_surviving_source() {
    type S = Model<6>;
    for birth in [false, true] {
        let host = install_definition::<S, TwoProgram>();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let body = create(&host, &request, 10, vec![20]);
        let owner = body
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S>>()
            .unwrap()
            .entity_id();
        settle(&host, &request, 10);
        let outcome = request
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
            .idempotency(&"before-foreign-writer".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        handler::reset_contacts();
        let _fault = host
            .runtime()
            .corrupt_next_coupled_participant_for_test(birth);
        let error = try_reverse(&request, &receipt, "foreign-writer-restore")
            .expect_err("a corrupt participant must never publish");
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::ForeignCoupledOwner),
            "birth={birth}: {error:?}"
        );
        assert_eq!(error.context.affected_records, [RecordRef::Entity(owner)]);
        assert_eq!(
            handler::contacts(),
            1,
            "the genuine installed writer ran before Native refusal"
        );
        for id in [10, 101, 103] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
        for id in [20, 102, 104] {
            assert!(request.query(query::Input { id }).execute().is_err());
        }
        settle(&host, &request, 10);
        assert_eq!(
            handler::contacts(),
            1,
            "refusal preserves the body's settled output"
        );
    }
}
