//! Bind a derived-relation read to the immutable Native pin.
use super::RelationalPinnedRead;
use contracts::derived_relation::definition::DerivedRelationHandle as Handle;
use contracts::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use contracts::derived_relation::read_shape::DerivedRelationReadShape;
use contracts::graph_member::key::NativeMemberKey;
use mutation::derived_relation::pinned::scope::PinnedDerivedRelationDenial;
use mutation::derived_relation::pinned::scope::PinnedDerivedRelationScope;
use mutation::derived_relation::pinned::PinnedDerivedRelationRead;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

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
        mutation::derived_relation::pinned::acquisition::read_projected_source(
            &projection,
            execution,
            scope,
        )
    }
}
