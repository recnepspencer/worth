//! An inverse writer cannot change a co-anchor's retained payload.
use super::{
    co_courts::{create, produce},
    correction_courts::install_program,
    lifecycle::*,
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn changed_co_anchor_payload_refuses_with_that_original_source_subject() {
    type S = Model<5>;
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    let co = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<co_roles::CoCreated<S, 0>>()
        .unwrap()
        .unwrap()
        .entity_id();
    produce(&host, &request, 10);
    produce(&host, &request, 20);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: true,
            co_anchors: vec![20],
            scope: 1,
            id: 10,
            descendants: vec![101, 102, 201, 202],
        })
        .without_source()
        .idempotency(&"retire-for-bad-inverse".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let source = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&source)
        .idempotency("changed-co-payload")
        .limits(limits)
        .prepare()
        .unwrap();
    let outcome = match preparation {
        WorthQueryApplicationCorrectionPreparation::Prepared(prepared) => {
            prepared.publish().unwrap()
        }
        WorthQueryApplicationCorrectionPreparation::Settled(outcome) => outcome,
        _ => panic!("the false inverse must finish with a typed refusal"),
    };
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let error = denial.authored_retirement_error().unwrap();
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(
            worth_relational::facade::publication::AuthoredRetirementRefusal::IncompleteRestoration
        )
    );
    assert_eq!(
        error.context.affected_records,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            co
        )]
    );
    for id in [10, 20, 101, 102, 201, 202] {
        assert!(
            matches!(
                request.query(query::Input { id }).execute(),
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
            ),
            "refused inverse recreated {id}"
        );
    }
}
