//! Zero request budgets stop real executing doors before reader or provider contact.
pub use super::installed_operation_fixture::{
    controlled_workflow_workspace, execution_resource_request, workflow_workspace, GeometryDomain,
    ReadFamily, WorkflowRead,
};
pub use std::num::NonZeroUsize;
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, foundation};
pub use worth_query_execution::facade::application_contribution::{
    WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationResourceDenial as Resource,
};
pub use worth_query_execution::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    installed_source_reads_on_this_thread_for_test as reads,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
pub struct Restore(
    pub Placement,
    pub Option<worth_foundational::ExecutionBudget>,
);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
pub fn refuse(zero_memory: bool, placement: Placement) -> Restore {
    Restore(
        place(placement),
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            if zero_memory { Some(0) } else { None },
            Some(if zero_memory { 8_000_000 } else { 0 }),
        ))),
    )
}
pub fn expected(zero_memory: bool, _placement: Placement) -> Denial {
    let roots = reports();
    assert_eq!(roots.len(), 1);
    let cause = *roots[0].as_ref().expect_err("refused opening");
    if !zero_memory {
        assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
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
pub fn bind(
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

pub mod artifact_roots;
pub mod direct_roots;
pub mod workflow_retry;
pub mod workflow_roots;

pub trait ProbeWorkspace {
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
pub fn verify<T, W: ProbeWorkspace>(
    name: &str,
    mut setup: impl FnMut() -> (W, T),
    mut execute: impl FnMut(T, &mut worth_query::facade::runtime::WorthQueryWorkspace) -> Option<Denial>,
) {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
