//! Bind a derived-relation read to the immutable Native pin.
use super::RelationalPinnedRead;
use crate::derived_relation::{
    DerivedRelationHandle as Handle, DerivedRelationReadDenial as Denial, DerivedRelationReadShape,
    PinnedDerivedRelationDenial, PinnedDerivedRelationRead, PinnedDerivedRelationScope,
};
use crate::graph_member::NativeMemberKey;
use worth_execution::ExecutionRequest;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub fn derived_relations_from(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        self.derived_source(
            execution,
            definition,
            root,
            DerivedRelationReadShape::Adjacency { reverse: false },
            maximum_work,
        )
    }
    /// Evaluate the exact registered recipe under the original request.
    pub fn derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        shape: DerivedRelationReadShape,
        maximum_work: Option<usize>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        self.read_derived_source(
            execution,
            PinnedDerivedRelationScope::new(definition, root, shape).with_work_limit(maximum_work),
        )
        .map_err(|stop| stop.kind)
    }
    /// Complete data retains its actual payer. Refusal retains actual reads/work
    /// without returning any partial successful data or replacement allowance.
    pub fn read_derived_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedDerivedRelationScope<'_>,
    ) -> Result<PinnedDerivedRelationRead, PinnedDerivedRelationDenial> {
        let projection = self.project_read();
        crate::derived_relation::read_projected_source(&projection, execution, scope)
    }
}
