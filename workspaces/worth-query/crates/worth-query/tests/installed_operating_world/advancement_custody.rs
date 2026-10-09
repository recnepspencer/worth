//! Zero request budgets stop real executing doors before reader or provider contact.
use super::installed_operation_fixture::{
    controlled_workflow_workspace, execution_resource_request, workflow_workspace, GeometryDomain,
    ReadFamily, WorkflowRead,
};
use std::num::NonZeroUsize;
use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation};
use worth_query_execution::facade::application_contribution::{
    WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationResourceDenial as Resource,
};
use worth_query_execution::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    installed_source_reads_on_this_thread_for_test as reads,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
fn refuse(zero_memory: bool, placement: Placement) -> Restore {
    Restore(
        place(placement),
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            if zero_memory { 0 } else { 64 * 1_024 * 1_024 },
            if zero_memory { 8_000_000 } else { 0 },
        ))),
    )
}
fn expected(zero_memory: bool, placement: Placement) -> Denial {
    let roots = reports();
    assert_eq!(roots.len(), 1);
    let cause = *roots[0].as_ref().expect_err("refused opening");
    if !zero_memory {
        assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
    } else if placement == Placement::Serial {
        assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
    } else {
        let Denial::Resource(Resource::MemoryLimit {
            level,
            requested,
            admitted,
        }) = cause
        else {
            panic!("exact lease bytes")
        };
        assert_eq!(level, worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
        assert!(requested > 0);
        assert_eq!(admitted, 0);
    }
    cause
}
fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}
#[test]
fn workflow_start_refuses_before_any_stage_or_reader() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let mut admitted_workspace = workflow_workspace("workflow-custody-control").unwrap();
        let admitted = bind(&admitted_workspace)
            .admit_workflow_resources(execution_resource_request(), &admitted_workspace)
            .unwrap();
        let before = reads();
        let control = admitted.start_workflow(&mut admitted_workspace);
        assert!(
            matches!(control, TransitionOutcome::Success(_)),
            "admitted workflow starts"
        );
        assert!(
            reads() > before,
            "admitted workflow start contacts its real source"
        );

        for zero_memory in [false, true] {
            let mut workspace = workflow_workspace("workflow-request-custody").unwrap();
            let admitted = bind(&workspace)
                .admit_workflow_resources(execution_resource_request(), &workspace)
                .unwrap();
            let _restore = refuse(zero_memory, placement);
            let before = reads();
            reports();
            let TransitionOutcome::Denied(denial) = admitted.start_workflow(&mut workspace) else {
                panic!("the root refuses before workflow preparation");
            };
            assert_eq!(
                denial.kind(),
                &domain::WorthQueryWorkflowStartDenialKind::ExecutionRequest(expected(
                    zero_memory,
                    placement
                ))
            );
            assert_eq!(denial.counters().stage_executor_contacts, 0);
            assert_eq!(reads(), before);
        }
    }
}
#[test]
fn projection_readmission_refuses_before_reading_the_pair() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let mut workspace = controlled_workflow_workspace("provider-request-readmission").unwrap();
        let prior = workspace.domain(GeometryDomain).unwrap();
        let (settled, _) = settle_workflow(&mut workspace);
        let live = promoted(settled, &mut workspace);
        workspace.advance_domain_installation_generation().unwrap();
        let (_, receipt) = workspace
            .rebind_domain(prior.rebind_request())
            .unwrap()
            .into_parts();
        let (candidate, _) = settle_workflow(&mut workspace);
        let candidate = candidate.into_lifecycle();
        let witness = live.rebind_witness_for(&candidate, receipt).unwrap();

        let before = reads();
        let control = live.rebind_with(candidate, witness, &mut workspace);
        assert!(
            reads() > before,
            "admitted projection readmission contacts its real source"
        );
        assert!(
            !matches!(
                control,
                domain::WorthQueryWorkflowProjectionRebindOutcome::Stopped(_)
            ),
            "admitted pair readmits"
        );

        use super::workflow_projection_lifecycle::{promoted, settle_workflow};
        for zero_memory in [false, true] {
            let mut workspace =
                controlled_workflow_workspace("provider-request-readmission").unwrap();
            let prior = workspace.domain(GeometryDomain).unwrap();
            let (settled, _) = settle_workflow(&mut workspace);
            let live = promoted(settled, &mut workspace);
            workspace.advance_domain_installation_generation().unwrap();
            let (_, receipt) = workspace
                .rebind_domain(prior.rebind_request())
                .unwrap()
                .into_parts();
            let (candidate, _) = settle_workflow(&mut workspace);
            let candidate = candidate.into_lifecycle();
            let witness = live.rebind_witness_for(&candidate, receipt).unwrap();
            let _restore = refuse(zero_memory, placement);
            let before = reads();
            reports();
            let domain::WorthQueryWorkflowProjectionRebindOutcome::Stopped(stop) =
                live.rebind_with(candidate, witness, &mut workspace)
            else {
                panic!("readmission cannot enter its provider under a refused root");
            };
            assert_eq!(
                stop.kind(),
                domain::WorthQueryProjectionTransitionDenialKind::ExecutionRequest(expected(
                    zero_memory,
                    placement
                ))
            );
            assert_eq!(stop.work().compatibility_readmissions(), 0);
            assert_eq!(stop.work().candidate().planning_attempts, 0);
            assert_eq!(reads(), before);
        }
    }
}

mod artifact_roots;
mod direct_roots;
mod workflow_retry;
mod workflow_roots;

pub(super) trait ProbeWorkspace {
    fn workspace_mut(&mut self) -> &mut worth_query::facade::runtime::WorthQueryWorkspace;
}
impl ProbeWorkspace for worth_query::facade::runtime::WorthQueryWorkspace {
    fn workspace_mut(&mut self) -> &mut Self {
        self
    }
}
impl ProbeWorkspace for worth_query::facade::consumer_kit::WorthQueryControlledTestWorkspace {
    fn workspace_mut(&mut self) -> &mut worth_query::facade::runtime::WorthQueryWorkspace {
        self
    }
}
pub(super) fn verify<T, W: ProbeWorkspace>(
    name: &str,
    mut setup: impl FnMut() -> (W, T),
    mut execute: impl FnMut(T, &mut worth_query::facade::runtime::WorthQueryWorkspace) -> Option<Denial>,
) {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        for budget in [None, Some(false), Some(true)] {
            bound(None);
            let (mut workspace, state) = setup();
            let _budget = budget.map(|memory| refuse(memory, placement));
            reports();
            let before = reads();
            let cause = execute(state, workspace.workspace_mut());
            if let Some(memory) = budget {
                assert_eq!(cause, Some(expected(memory, placement)), "{name}");
                assert_eq!(reads(), before, "{name}: refused before its reader");
            } else {
                assert_eq!(cause, None, "{name}: admitted control");
                assert!(
                    reads() > before,
                    "{name}: same admitted entry moves its reader"
                );
                assert_eq!(reports().len(), 1, "{name}: one public call opens once");
            }
        }
    }
}
