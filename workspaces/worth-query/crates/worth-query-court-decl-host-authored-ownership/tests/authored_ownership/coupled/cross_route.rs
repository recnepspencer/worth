//! Sharing a commit coordinate does not share a producer's write custody.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, Graph, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};
#[test]
fn one_routes_preserve_cannot_cover_another_routes_current_member() {
    type S = Model<61>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    let other = create(&host, &request, 11, vec![]);
    let owner = other
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 20,
            co_anchors: vec![],
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"share-publication-coordinate".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    let publication = receipt.commit_reference().commit_id;
    // Both writers now have the same current commit, but distinct routes and members.
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: false,
            co_anchors: vec![],
            scope: 10,
            id: 30,
            depth: 0,
        })
        .without_source()
        .idempotency(&"change-first-body-only".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let mut pending = request
        .demand(program::Demand(10))
        .retain_program_outputs::<TwoProgram, Graph<S>>(&host)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    let error = loop {
        match pending.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                break original(&denial)
            }
            WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                panic!("foreign deletion was admitted")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                let error = original(terminal.denial());
                terminal
                    .close(&request)
                    .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                break error;
            }
        }
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner })
    );
    assert_eq!(
        error.context.affected_records.last(),
        Some(&RecordRef::Entity(owner))
    );
    for id in [101, 103, 111, 113] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    assert!(request.query(query::Input { id: 104 }).execute().is_err());
}

fn original(
    denial: &WorthQueryProgramOutputAdvanceDenial,
) -> worth_relational::facade::publication::PublicationError {
    let WorthQueryProgramOutputAdvanceDenial::Outputs(
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(denial),
        ),
    ) = denial
    else {
        panic!("the original producer refusal must be preserved: {denial:?}")
    };
    denial
        .authored_retirement_error()
        .expect("typed Native custody refusal")
        .clone()
}
