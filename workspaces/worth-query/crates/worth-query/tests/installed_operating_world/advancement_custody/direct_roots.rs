//! Direct execution and each direct projection root keep custody ahead of source reads.
use super::super::installed_operation_fixture::{
    configured_runtime, ReadExecutionInput, ReadVertex,
};
use super::*;
type Workspace = worth_query::facade::runtime::WorthQueryWorkspace;
type Current = domain::WorthQueryCurrentDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;
type Live = domain::WorthQueryLiveBoundDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

fn workspace() -> Workspace {
    configured_runtime()
        .workspace("direct-root-custody")
        .unwrap()
}
fn admitted(
    workspace: &Workspace,
) -> worth_query::facade::installed::operation::WorthQueryAdmittedDirectOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap()
        .admit_execution_resources(
            ReadExecutionInput::default(),
            execution_resource_request(),
            workspace,
        )
        .unwrap()
}
fn current(workspace: &mut Workspace) -> Current {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            execution_resource_request(),
            workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
        .into_lifecycle()
}
fn promoted(current: Current, workspace: &mut Workspace) -> Live {
    match current.promote(workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("honest direct projection promotes"),
    }
}

#[test]
fn direct_execute_opens_before_its_reader() {
    verify(
        "direct execute",
        || {
            let workspace = workspace();
            let operation = admitted(&workspace);
            (workspace, operation)
        },
        |operation, workspace| match operation.execute(workspace) {
            TransitionOutcome::Success(_) => None,
            TransitionOutcome::Denied(denial) => match denial.kind() {
                domain::WorthQueryBoundExecutionDenialKind::ExecutionRequest(cause) => Some(*cause),
                other => panic!("unexpected direct denial: {other:?}"),
            },
            _ => panic!("unexpected direct execution posture"),
        },
    );
}

#[test]
fn direct_promotion_opens_before_its_reader() {
    verify(
        "direct promotion",
        || {
            let mut workspace = workspace();
            let current = current(&mut workspace);
            (workspace, current)
        },
        |current, workspace| match current.promote(workspace) {
            domain::WorthQueryProjectionPromotionOutcome::Promoted(_) => None,
            domain::WorthQueryProjectionPromotionOutcome::Denied(stop) => match stop.kind() {
                domain::WorthQueryProjectionPromotionDenialKind::ExecutionRequest(cause) => {
                    Some(cause)
                }
                other => panic!("unexpected direct promotion denial: {other:?}"),
            },
            _ => panic!("unexpected direct promotion posture"),
        },
    );
}

#[test]
fn direct_replacement_opens_before_its_reader() {
    verify(
        "direct replacement",
        || {
            let mut workspace = workspace();
            let current = current(&mut workspace);
            let live = promoted(current, &mut workspace);
            let candidate = self::current(&mut workspace);
            let witness = live.replacement_witness_for(&candidate).unwrap();
            (workspace, (live, candidate, witness))
        },
        |(live, candidate, witness), workspace| match live
            .replace_with(candidate, witness, workspace)
        {
            domain::WorthQueryProjectionReplacementOutcome::Replaced(_) => None,
            domain::WorthQueryProjectionReplacementOutcome::Stopped(stop) => match stop.kind() {
                domain::WorthQueryProjectionTransitionDenialKind::ExecutionRequest(cause) => {
                    Some(cause)
                }
                other => panic!("unexpected direct replacement denial: {other:?}"),
            },
            _ => panic!("unexpected direct replacement posture"),
        },
    );
}

#[test]
fn direct_rebinding_opens_before_its_reader() {
    verify(
        "direct rebinding",
        || {
            let mut workspace = configured_runtime()
                .controlled_workspace("direct-rebind-custody")
                .unwrap();
            let prior = workspace.domain(GeometryDomain).unwrap();
            let current = current(&mut workspace);
            let live = promoted(current, &mut workspace);
            workspace.advance_domain_installation_generation().unwrap();
            let (_, receipt) = workspace
                .rebind_domain(prior.rebind_request())
                .unwrap()
                .into_parts();
            let candidate = self::current(&mut workspace);
            let witness = live.rebind_witness_for(&candidate, receipt).unwrap();
            (workspace, (live, candidate, witness))
        },
        |(live, candidate, witness), workspace| match live
            .rebind_with(candidate, witness, workspace)
        {
            domain::WorthQueryProjectionRebindOutcome::Rebound(_) => None,
            domain::WorthQueryProjectionRebindOutcome::Stopped(stop) => match stop.kind() {
                domain::WorthQueryProjectionTransitionDenialKind::ExecutionRequest(cause) => {
                    Some(cause)
                }
                other => panic!("unexpected direct rebind denial: {other:?}"),
            },
            _ => panic!("unexpected direct rebinding posture"),
        },
    );
}
