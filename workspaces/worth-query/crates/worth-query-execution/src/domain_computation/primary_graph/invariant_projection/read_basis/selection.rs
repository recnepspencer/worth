//! Equality selections at a committed root or its actual applied successor.
use super::*;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::indexes::*;
pub(in crate::domain_computation::primary_graph::invariant_projection) struct ProjectionEntitySelection
{
    definition: RetainedDerivedIndexDefinition,
    candidates: Vec<EntityId>,
    examined: usize,
    overflowed: bool,
}
impl ProjectionEntitySelection {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn retain_definition(
        &self,
    ) -> RetainedDerivedIndexDefinition {
        self.definition.clone()
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn candidate_entity_ids(
        &self,
    ) -> &[EntityId] {
        &self.candidates
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn into_candidate_entity_ids(
        self,
    ) -> Vec<EntityId> {
        self.candidates
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn examined_entry_count(
        &self,
    ) -> usize {
        self.examined
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn overflowed(
        &self,
    ) -> bool {
        self.overflowed
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn committed(
        value: BoundedEntityFieldLookupOutcome,
    ) -> Self {
        Self {
            definition: value.retain_definition(),
            examined: value.examined_entry_count(),
            overflowed: value.overflowed(),
            candidates: value.into_candidate_entity_ids(),
        }
    }
}
pub(in crate::domain_computation::primary_graph::invariant_projection) enum ProjectionSelectionStop
{
    Committed(BoundedEntityFieldLookupDenial),
    Applied(AppliedEntityFieldLookupDenial),
}
impl ProjectionSelectionStop {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn examined_entry_count(
        &self,
    ) -> usize {
        match self {
            Self::Committed(e) => e.examined_entry_count(),
            Self::Applied(AppliedEntityFieldLookupDenial::PredecessorIndex(e)) => {
                e.examined_entry_count()
            }
            _ => 0,
        }
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn kind(
        &self,
    ) -> BoundedEntityFieldLookupDenialKind {
        use BoundedEntityFieldLookupDenialKind as K;
        match self {
            Self::Committed(e) => e.kind(),
            Self::Applied(e) => match e {
                AppliedEntityFieldLookupDenial::Execution(e) => K::Execution(*e),
                AppliedEntityFieldLookupDenial::PredecessorIndex(e) => e.kind(),
                AppliedEntityFieldLookupDenial::InvalidCandidateLimit => K::InvalidCandidateLimit,
                AppliedEntityFieldLookupDenial::ForeignPredecessor
                | AppliedEntityFieldLookupDenial::ExactBasisRequired => K::SnapshotUnavailable,
            },
        }
    }
}
impl NativeProjectionRead<'_> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn select_entities(
        &self,
        request: BoundedEntityFieldLookupRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ProjectionEntitySelection, ProjectionSelectionStop> {
        match self {
            Self::Committed(runtime) => runtime
                .index_access()
                .execute_bounded_entity_field_lookup(
                    request,
                    BoundedIndexParityMode::Production,
                    execution,
                )
                .map(ProjectionEntitySelection::committed)
                .map_err(ProjectionSelectionStop::Committed),
            Self::Applied {
                before,
                candidate,
                own_outputs,
            } => {
                let prepared = before
                    .prepare_entity_field_lookup(
                        request.index_id(),
                        request.entity_kind(),
                        request.field_locator(),
                        execution,
                    )
                    .map_err(ProjectionSelectionStop::Committed)?;
                ProjectionEntitySelection::applied_with_own_outputs(
                    candidate,
                    *own_outputs,
                    &prepared,
                    request.value(),
                    request.candidate_limit(),
                    execution,
                )
            }
        }
    }
}

impl ProjectionEntitySelection {
    /// An applied selection seen by a successor round (§1.5): records its own
    /// chain wrote exclusively match as they did before the chain, every
    /// other record as it does now.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn applied_with_own_outputs(
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own_outputs: Option<&super::OwnOutputOverlay>,
        prepared: &PreparedEntityFieldLookup,
        value: &AspectValue,
        limit: usize,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, ProjectionSelectionStop> {
        let current = Self::applied(candidate, prepared, value, limit, execution)?;
        let Some(own) = own_outputs else {
            return Ok(current);
        };
        let head = Self::applied(own.head(), prepared, value, limit, execution)?;
        let mut candidates = current
            .candidates
            .into_iter()
            .filter(|entity| !own.covers(*entity))
            .chain(
                head.candidates
                    .into_iter()
                    .filter(|entity| own.covers(*entity)),
            )
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.dedup();
        Ok(Self {
            definition: current.definition,
            examined: current.examined.saturating_add(head.examined),
            overflowed: current.overflowed || head.overflowed || candidates.len() > limit,
            candidates,
        })
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn applied(
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        prepared: &PreparedEntityFieldLookup,
        value: &AspectValue,
        limit: usize,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, ProjectionSelectionStop> {
        let value = candidate
            .lookup_applied_entity_field(
                execution,
                prepared,
                value,
                BoundedIndexParityMode::Production,
                limit,
            )
            .map_err(ProjectionSelectionStop::Applied)?;
        Ok(Self {
            definition: prepared.retain_definition(),
            examined: value.examined_entry_count(),
            overflowed: value.overflowed(),
            candidates: value.into_candidate_entity_ids(),
        })
    }
}
