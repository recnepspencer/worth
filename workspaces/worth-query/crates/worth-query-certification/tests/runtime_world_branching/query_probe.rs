use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph, product,
};

use crate::product_query_support::{controls, principal, product_identity};
use crate::schema::*;
use crate::world::{self, CourtroomWorld};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct QueryWork {
    pub basis_acquisitions: usize,
    pub admission_product_resolutions: usize,
    pub execution_product_resolutions: usize,
    pub fallback_count: usize,
}

pub(super) struct QueryObservation {
    pub input: String,
    pub identity: product::WorthQueryProductBranchReadIdentity,
    pub work: QueryWork,
}

pub(super) fn read(
    world: &CourtroomWorld,
    branch: product::WorthQueryProductBranch,
) -> QueryObservation {
    read_across(world, branch, || ()).0
}

pub(super) fn read_across<Interleaved>(
    world: &CourtroomWorld,
    branch: product::WorthQueryProductBranch,
    interleave: impl FnOnce() -> Interleaved,
) -> (QueryObservation, Interleaved) {
    let request = world::request_scope();
    let principal = principal(world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .expect("the installed ordinary query must remain available");
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let selected = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the modeled product must remain selectable")
        })
        .expect("installed host call opens");
    let scope = world
        .application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the selected product must resolve its intent")
        })
        .expect("installed host call opens");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .application
        .with_host_advancement(|host_phase| {
            selected
                .admit_application_query(
                    &host_phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls(&request),
                )
                .expect("the ordinary read must be admitted on the selected product")
        })
        .expect("installed host call opens");
    let after_admission = observer.observe();
    let interleaved = interleave();
    let before_execution = observer.observe();
    let result = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .execute_application_query_one_shot(&host_phase, plan)
                .expect("the admitted exact-basis read must execute")
        })
        .expect("installed host call opens");
    let receipt = result.receipt();
    assert!(receipt.basis_released());
    let authorization = receipt.authorization_work();
    let after = observer.observe();
    let observation = QueryObservation {
        input: result.rows()[0].input.clone(),
        identity: product_identity(receipt).clone(),
        work: QueryWork {
            basis_acquisitions: (after_admission.acquisitions() - before.acquisitions())
                + (after.acquisitions() - before_execution.acquisitions()),
            admission_product_resolutions: authorization.admission_security_product_resolutions(),
            execution_product_resolutions: authorization.execution_security_product_resolutions(),
            fallback_count: receipt.fallback_count(),
        },
    };
    assert_eq!(after.active() + 1, before_execution.active());
    (observation, interleaved)
}

pub(super) fn assert_ordinary_work(work: QueryWork) {
    // Fresh security stages reuse the read's exact snapshot, so one basis
    // acquisition serves admission and execution.
    assert_eq!(work.basis_acquisitions, 1);
    assert_eq!(work.admission_product_resolutions, 1);
    assert_eq!(work.execution_product_resolutions, 1);
    assert_eq!(work.fallback_count, 0);
}

pub(super) fn assert_retained_work(work: QueryWork) {
    assert_ordinary_work(work);
}
