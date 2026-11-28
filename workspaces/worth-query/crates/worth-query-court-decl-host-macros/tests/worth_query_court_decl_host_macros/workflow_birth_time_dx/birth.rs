//! One workflow birth through the public request API: prepare the native
//! requirement, capture the original creator, bind it to that requirement,
//! retain its declared SourceOnly action and advance it with fresh authority.
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

use super::creator::{CreatedRequest, OpenRequest};
use super::program::{owner, BirthProgram, Caller, OpenRequestAction};
use super::schema::{ReadRequest, RequestView};
use super::workflow::{RequestWorkflow, WorkflowHome, World};
use super::COMPANY;

/// Births request `request` and returns the instance its own commit started.
pub fn birth(world: &World, request: u64) -> PublishedWorkflowInstanceRef {
    let runtime = world.home.runtime().runtime();
    let caller = owner(runtime.installed_schema());
    let open = OpenRequest {
        company: COMPANY.to_owned(),
        request,
    };
    let key = request;
    let required = runtime
        .request(&caller.principal, &caller.scope)
        .on_branch(world.definition.branch())
        .advanced()
        .mutate(open.clone())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_birth::<RequestWorkflow, CreatedRequest>(
            world.home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            world.definition.clone(),
        )
        .unwrap_or_else(|denial| panic!("birth prepares: {denial:?}"));
    let fresh = runtime
        .request(&caller.principal, &caller.scope)
        .on_branch(required.branch());
    let Ok(original) = fresh.mutate(open).without_source().idempotency_owned(key) else {
        panic!("the original creator captures its input");
    };
    let operation = original
        .for_workflow_birth::<RequestWorkflow, CreatedRequest>(
            world.home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            &required,
        )
        .unwrap_or_else(|(_, denial)| panic!("birth binds: {denial:?}"))
        .retain_operation::<BirthProgram, OpenRequestAction>(world.home.runtime())
        .unwrap_or_else(|(_, denial)| panic!("birth retains: {denial:?}"));
    let done = match operation.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("birth refused: {denial:?}"),
        WorthQuerySourceOnlyAdvance::Progress(_) => panic!("birth did not complete"),
        WorthQuerySourceOnlyAdvance::Terminal(_) => panic!("birth ended terminally"),
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = done.evidence() else {
        panic!("a first birth commits fresh");
    };
    // The receipt records exactly one native instance for this exact birth.
    let instance = required
        .instance_in(receipt)
        .unwrap_or_else(|denial| panic!("the birth commit started its instance: {denial:?}"));
    if done.close(&fresh).is_err() {
        panic!("the completed birth releases its cleanup");
    }
    instance
}

/// Births `request` in `company` on `definition` as `caller`, executing the
/// bound creator in the program as a Directory Home requires.
pub fn birth_in(
    home: &WorkflowHome,
    definition: &PublishedWorkflowDefinitionRef,
    caller: &Caller,
    company: &str,
    request: u64,
) -> PublishedWorkflowInstanceRef {
    let open = OpenRequest {
        company: company.to_owned(),
        request,
    };
    let runtime = home.runtime().runtime();
    let fresh = || runtime.request(&caller.principal, &caller.scope).advanced();
    let required = fresh()
        .mutate(open.clone())
        .without_source()
        .idempotency(&request)
        .prepare_workflow_birth::<RequestWorkflow, CreatedRequest>(
            home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            definition.clone(),
        )
        .unwrap_or_else(|denial| panic!("birth prepares: {denial:?}"));
    let outcome = fresh()
        .mutate(open)
        .without_source()
        .idempotency(&request)
        .for_workflow_birth::<RequestWorkflow, CreatedRequest>(
            home.workflow_vocabulary::<RequestWorkflow>().unwrap(),
            &required,
        )
        .unwrap_or_else(|denial| panic!("birth binds: {denial:?}"))
        .execute_capability_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_or_else(|denial| panic!("birth executes: {denial:?}"));
    let Some(receipt) = outcome.receipt() else {
        panic!("the birth commits");
    };
    required
        .instance_in(receipt)
        .unwrap_or_else(|denial| panic!("the birth commit started its instance: {denial:?}"))
}

/// A request as committed, read back by the owner.
pub fn read(world: &World, request: u64) -> RequestView {
    let runtime = world.home.runtime().runtime();
    let caller = owner(runtime.installed_schema());
    let mut rows = runtime
        .request(&caller.principal, &caller.scope)
        .query(ReadRequest { request })
        .execute()
        .unwrap_or_else(|denial| panic!("owner reads the request: {denial:?}"))
        .into_rows();
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}
