//! A foreign fresh source is the actual declared subject, not its retired predecessor.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn an_extra_fresh_correction_root_names_the_unpublished_source() {
    type S = Model<24>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = create(&host, &request, 10, vec![20]);
    let original = original.outputs_of::<CreatedOutputs>().unwrap();
    let source = original.entity::<Created<S>>().unwrap().entity_id();
    let stage = original
        .entity::<super::super::co_roles::CoCreated<S, 0>>()
        .unwrap()
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
        .idempotency(&"retire-before-extra-fresh-root".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    super::producer::handler::reset_contacts();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("extra-fresh-root")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("correction must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("typed extra owner");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ExtraCoupledOwner)
    );
    let [worth_relational::facade::transactions::RecordRef::Entity(subject)] =
        error.context.affected_records.as_slice()
    else {
        panic!("{error:?}")
    };
    assert_ne!(
        *subject, stage,
        "the declaration selected the fresh stage, not the retired original"
    );
    assert_ne!(
        *subject, source,
        "the real body source was not the extra participant"
    );
    assert_eq!(subject.partition_value(), stage.partition_value());
    assert_eq!(super::producer::handler::contacts(), 0);
    for id in [10, 101, 103] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    for id in [20, 102, 104] {
        assert!(request.query(query::Input { id }).execute().is_err());
    }
    settle(&host, &request, 10);
    assert_eq!(
        super::producer::handler::contacts(),
        0,
        "a refused correction preserves the settled body"
    );
}
