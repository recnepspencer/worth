//! Complete target execution returns only consistency and exact target-pin custody.
use super::super::{
    invocation::controlled, source::service::ManagedSourceRoot,
    WorthQueryManagedIndexStop as ManagedStop, WorthQueryManagedIndexWorkDenial as WorkDenial,
};
use super::*;
use crate::domain_computation::primary_graph::{
    access_strategy::{
        coverage::CoverageFrame,
        realized_footprint::{
            CapturedManagedInfluence, RetainedManagedInfluence, RevalidatedManagedSource,
        },
    },
    application_attempt::WorthQuerySourceCurrentnessFailure as Stop,
    application_query::one_shot::outcome::material::MaterialCompletion,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    runtime::{DerivedRelationReadDenial, ManagedIndexSourcePreparationDenial, RelationalRuntime},
    snapshots::SnapshotHandle,
};

impl<S: ApplicationSchema + 'static, I: ApplicationManagedIndexBinding<S>> RetainedManagedInfluence
    for CapturedIndexInfluence<S, I>
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn comparison_work(&self) -> u64 {
        self.parameters.retained_bytes
    }
    fn same_inputs(&self, other: &dyn RetainedManagedInfluence) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        self.scope == other.scope
            && self._admitted_source.principal() == other._admitted_source.principal()
            && self._admitted_source.runtime() == other._admitted_source.runtime()
            && self._admitted_source.scope() == other._admitted_source.scope()
            && Arc::ptr_eq(&self.adapter, &other.adapter)
            && self
                .parameters
                .parent()
                .same_association(other.parameters.parent())
            && self.parameters.parameters().bindings() == other.parameters.parameters().bindings()
    }
    fn revalidate(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        audience: &CapturedManagedInfluence,
    ) -> Result<Option<RevalidatedManagedSource>, Stop> {
        let original = self
            .original
            .payload()
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Stop::Unavailable)?;
        let observation = match runtime.read_truth().observe_managed_index_source(
            execution,
            snapshot,
            original.rows.program(),
            self.scope,
        ) {
            Ok(observation) => observation,
            // Retirement or eviction proves no equality. Preserve the ordinary
            // conservative stale result; absence is never an empty candidate cut.
            Err(ManagedIndexSourcePreparationDenial::RootUnavailable) => return Ok(None),
            Err(cause) => return Err(native_stop(cause)),
        };
        if !self.original.same_source_scope(&observation) {
            return Err(Stop::ManagedIndex(ManagedStop::ForeignCoverage.into()));
        }
        let root = observation
            .payload()
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Stop::Unavailable)?;
        if !root.installed.same_installation(self.adapter.installed())
            || !Arc::ptr_eq(&root.adapter, &self.adapter)
            || !root.installed.admits_association(self.parameters.parent())
        {
            return Err(Stop::ManagedIndex(ManagedStop::ForeignCoverage.into()));
        }
        let frame = CoverageFrame::retained(
            root.rows.view(&root.bindings, None),
            &root.nodes,
            root,
            self.parameters.raw(),
            &observation,
            audience,
            self.parameters
                .parent()
                .access_contract()
                .meaning()
                .match_posture(),
        );
        // The original cut and all candidate bodies were captured once. Only
        // this target executes again; it independently proves negative coverage.
        let selected = self
            .adapter
            .candidates(execution, frame)
            .map_err(work_stop)?;
        let selected = controlled(execution, |control| {
            selected.receive(frame, self.adapter.coverage_authority(), true, control)
        })
        .map_err(work_stop)?;
        if !controlled(execution, |control| {
            self.candidates.matches(&root.rows, &selected, control)
        })
        .map_err(work_stop)?
        {
            return Ok(None);
        }
        let checked = CheckedManagedFootprintCapture {
            frame: worth_proof::BorrowedRealizedFootprintFrame::new(
                self.parameters.parent(),
                root,
                self.parameters.parameters(),
                audience,
            ),
            coverage: selected.coverage().ok_or(Stop::Unavailable)?,
        };
        let proof = self
            .adapter
            .verify_realized(&checked)
            .map_err(|cause| work_stop(cause.into()))?;
        let allocation = SourceFactsAllocation::open(execution).map_err(Stop::Execution)?;
        MaterialCompletion::new(execution, &allocation)
            .arc::<ManagedIndexSourceObservation>()
            .map_err(Stop::Execution)?;
        execution
            .consult()
            .map_err(|cause| Stop::Execution(cause.into()))?;
        Ok(Some(RevalidatedManagedSource {
            observation: Arc::new(observation),
            proof,
            allocation,
        }))
    }
}

fn native_stop(cause: ManagedIndexSourcePreparationDenial) -> Stop {
    match cause {
        ManagedIndexSourcePreparationDenial::Source(DerivedRelationReadDenial::Request(cause)) => {
            Stop::Execution(cause)
        }
        _ => Stop::Unavailable,
    }
}
fn work_stop(cause: WorkDenial) -> Stop {
    match cause {
        WorkDenial::Controlled(ManagedStop::Request(cause)) => Stop::Execution(cause.into()),
        WorkDenial::Controlled(ManagedStop::MaterialStorage(cause)) => Stop::Execution(cause),
        other => Stop::ManagedIndex(other),
    }
}
