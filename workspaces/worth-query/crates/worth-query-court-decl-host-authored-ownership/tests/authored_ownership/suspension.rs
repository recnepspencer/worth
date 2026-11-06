//! Physical suspension cannot sever the custody of a source-owned publication.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    branch::RelationalMaterializationError, publication::AuthoredRetirementRefusal,
    transactions::TransactionCommitError,
};
#[test]
fn owned_suspension_refuses_and_independent_outputs_still_suspend_and_rematerialize() {
    run::<6>();
    run::<65>();
}
fn run<const MODE: u8>() {
    type S<const M: u8> = Model<M>;
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let body = create(
        &host,
        &request,
        10,
        if MODE == 6 { vec![20] } else { vec![] },
    );
    settle(&host, &request, 10);
    let source = request
        .query(query::Input { id: 10 })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let selected = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(body.product_branch())
                .select(&phase)
        })
        .unwrap()
        .unwrap();
    let result =
        selected.suspend_current_generated_output::<producer::Producer<S<MODE>>>(&scope, source);
    if MODE == 6 {
        let error = result
            .err()
            .expect("owned payload must remain materialized");
        let WorthQueryGeneratedOutputSuspensionFailure::Preparation(
            RelationalMaterializationError::Commit(TransactionCommitError::Publication {
                error,
                ..
            }),
        ) = error
        else {
            panic!("{error:?}")
        };
        assert!(
            matches!(
                error.authored_retirement_refusal(),
                Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { .. })
            ),
            "{error:?}"
        );
        for id in [10, 20, 101, 102] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
        settle(&host, &request, 10);
        return;
    }
    let suspended = result.unwrap_or_else(|e| panic!("independent suspension: {e:?}"));
    let mut reconstruction = host
        .with_host_advancement(|phase| {
            host.runtime()
                .reconstruct_generated_output::<producer::Producer<S<MODE>>>(&phase, suspended)
        })
        .unwrap()
        .unwrap_or_else(|_| panic!("independent reconstruction"));
    let a = reconstruction.output::<producer::Role<S<MODE>>>().unwrap();
    let b = reconstruction.output::<producer::Cell<S<MODE>>>().unwrap();
    let generated = |value| match value {
        WorthQueryReconstructedOutputEntity::Generated(value) => value,
        _ => panic!("selected payload is generated"),
    };
    for (id, entity) in [(101, generated(a)), (102, generated(b))] {
        reconstruction
            .field(&entity, schema::id::<S<MODE>>(), id)
            .unwrap();
        reconstruction
            .field(&entity, schema::value::<S<MODE>>(), 39)
            .unwrap();
    }
    let completed = reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("all actual payload fields restored"));
    host.runtime()
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|_| panic!("independent rematerialization"));
    for id in [101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
}
