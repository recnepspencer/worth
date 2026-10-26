use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::LocatorAuthority;
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;
use worth_relational::facade::runtime::RelationalFieldRevision;

use crate::domain_computation::primary_graph::application_attempt::reobserve_indexed_entity_selection;
use crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial;
use crate::domain_computation::primary_graph::application_attempt::Movement;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;

use super::adjacency;
use super::retirement;
use super::RebaseVerificationReason;
use super::WorthQueryApplicationObservedFact as Fact;

/// A fact a source query read. It stays at the revision it was read at, so
/// it is the one kind the walk asks the committed effect about: every other
/// fact is observed again at the committed snapshot. A field an owner call
/// read and the effect replaced is marked moved before the walk.
pub(super) fn reads_source(fact: &Fact) -> bool {
    matches!(
        fact,
        Fact::SourcePublication { .. }
            | Fact::StructuralWalk { .. }
            | Fact::Aggregate { .. }
            | Fact::SourceEntity { .. }
            | Fact::Entity { .. }
            | Fact::SourceFieldRevision { .. }
            | Fact::OptionalAspectAbsent { .. }
    )
}

/// One selected-snapshot resolution paired by ordinal with the original fact.
/// All resolutions are acquired before any original is consumed.
pub(super) enum PreparedFactRebase {
    Keep,
    /// A fact a source query read stays at the revision it read. The
    /// committed effect itself moved it, so the settlement is born stale here.
    KeepSuperseded,
    Field {
        locator: AspectFieldLocator,
        revision: RelationalFieldRevision,
    },
    Aspect(Option<u64>),
    Adjacency(Option<VersionId>),
    Replace(Fact),
}

impl PreparedFactRebase {
    /// Every resolution pays before it reads. A producer's source read
    /// reserves its own comparison; every other resolution is one unit, and
    /// an indexed probe one more. The request meter's stop is the outer
    /// error; the inner one is a fact the walk could not resolve.
    pub(super) fn prepare(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        retired: &std::collections::BTreeSet<worth_relational::facade::identity::EntityId>,
        producer_output: bool,

        prepared_endpoints: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
        >,
    ) -> Result<Result<Self, RebaseVerificationReason>, CompanionPreflightStop> {
        let unavailable = RebaseVerificationReason::NativeRevisionUnavailable;
        if let Some(entity_id) = retirement::retired_anchor(fact, retired) {
            // No other fact may stand in for a retirement the snapshot lacks.
            return Ok(
                retirement::committed_retirement(runtime, snapshot, entity_id, fact)
                    .map(Self::Replace)
                    .ok_or(unavailable),
            );
        }
        if producer_output
            && let Fact::OptionalAspectAbsent {
                entity_id,
                kind,
                locator,
            } = fact
        {
            match crate::domain_computation::primary_graph::application_attempt::observe_aspect_absence(
                runtime, execution, snapshot, *entity_id, *kind, locator,
            ) {
                Err(cause) => return Ok(Err(RebaseVerificationReason::Execution(cause))),
                Ok(Some(true)) => {},
                Ok(Some(false)) => return Ok(Ok(Self::KeepSuperseded)),
                Ok(None) => return Ok(Err(unavailable)),
            }
        }
        if producer_output
            && reads_source(fact)
            && !matches!(fact, Fact::OptionalAspectAbsent { .. })
        {
            return Ok(
                match fact.source_currentness_in(runtime, execution, snapshot)? {
                    Err(WorthQuerySourceCurrentnessFailure::OutputObservation(stop)) => match stop {
                        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop::Execution(cause) => Err(RebaseVerificationReason::Execution(cause)),
                        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop::Interrupted(event) => return Err(CompanionPreflightStop::Interrupted(event)),
                        _ => Err(unavailable),
                    },
                    Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
                        Err(RebaseVerificationReason::Execution(cause))
                    }
                    Ok(movement) => Ok(match movement.movement() {
                        Movement::Unmoved => Self::Keep,
                        Movement::Moved => Self::KeepSuperseded,
                    }),
                    Err(
                        WorthQuerySourceCurrentnessFailure::Unavailable
                        | WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded,
                    ) => Err(unavailable),
                },
            );
        }

        Ok(Self::resolve(
            runtime,
            execution,
            snapshot,
            fact,
            producer_output,
            prepared_endpoints,
        ))
    }

    fn resolve(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        producer_output: bool,

        prepared_endpoints: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
        >,
    ) -> Result<Self, RebaseVerificationReason> {
        let unavailable = RebaseVerificationReason::NativeRevisionUnavailable;
        match fact {
            Fact::OptionalAspectAbsent {
                entity_id, locator, ..
            } => {
                let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
                    return Err(unavailable);
                };
                view.entity_aspect_version(*entity_id, locator.aspect().aspect_key())
                    .map(Self::Aspect)
                    .ok_or(unavailable)
            }
            Fact::AspectPresence {
                entity_id,
                kind,
                aspect,
                ..
            } => {
                let present = crate::domain_computation::primary_graph::application_attempt::observe_aspect_presence(
                    runtime, execution, snapshot, *entity_id, *kind, aspect,
                ).map_err(RebaseVerificationReason::Execution)?.ok_or(unavailable)?;
                Ok(Self::Replace(Fact::AspectPresence {
                    entity_id: *entity_id,
                    kind: *kind,
                    aspect: aspect.clone(),
                    present,
                }))
            }
            Fact::Field {
                entity_id, locator, ..
            }
            | Fact::AbsentField {
                entity_id, locator, ..
            } => {
                let locator = AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    locator.aspect().aspect_key().clone(),
                    locator.field_path().clone(),
                );
                let revision = match runtime.read_truth().project_snapshot(snapshot) {
                    Some(view) => view
                        .entity_field_revision(execution, *entity_id, &locator)
                        .map_err(RebaseVerificationReason::Execution)?,
                    None => None,
                };
                match revision {
                    Some(revision) => Ok(Self::Field { locator, revision }),
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::SourceAspectRevision {
                entity_id, aspect, ..
            } => {
                match runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_aspect_version(*entity_id, aspect))
                {
                    Some(revision) => Ok(Self::Aspect(revision)),
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                comparison_work_limit,
                ..
            } => {
                let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
                    return if producer_output {
                        Err(unavailable)
                    } else {
                        Ok(Self::Keep)
                    };
                };
                match view.bounded_adjacency_structural_revision(
                    execution,
                    *anchor,
                    *relation_kind,
                    *direction,
                    *comparison_work_limit,
                ) {
                    Err(AdjacencyStructuralRevisionDenial::Execution(cause)) => {
                        Err(RebaseVerificationReason::Execution(cause))
                    }
                    Ok(comparison) => Ok(Self::Adjacency(comparison.revision())),
                    // The request paid the lookup above; a refusal here is
                    // the fact's own recorded limit, or a missing anchor or
                    // basis, so the fact is not resolved.
                    Err(
                        AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                        | AdjacencyStructuralRevisionDenial::AnchorUnavailable
                        | AdjacencyStructuralRevisionDenial::BasisUnavailable,
                    ) if producer_output => Err(unavailable),
                    Err(
                        AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                        | AdjacencyStructuralRevisionDenial::AnchorUnavailable
                        | AdjacencyStructuralRevisionDenial::BasisUnavailable,
                    ) => Ok(Self::Keep),
                }
            }
            Fact::Relation { .. } | Fact::Adjacency { .. } => {
                match adjacency::prepare_decision_adjacency(
                    execution,
                    runtime,
                    snapshot,
                    fact,
                    prepared_endpoints,
                )
                .map_err(RebaseVerificationReason::Execution)?
                {
                    Some(rebased) => Ok(Self::Replace(rebased)),
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::IndexedEntitySelection {
                candidate_limit, ..
            } => {
                // The selection is observed again at the committed snapshot.
                // The width reserves before probing and settles actual work,
                // including entries examined by a failed native lookup.
                let mut remaining = candidate_limit.saturating_add(1);
                let observed = reobserve_indexed_entity_selection(
                    fact,
                    runtime,
                    execution,
                    snapshot,
                    &mut remaining,
                );
                match observed {
                    Ok(observed) => Ok(Self::Replace(observed)),
                    Err(IndexedSelectionReobserveDenial::Lookup(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause)))=>Err(RebaseVerificationReason::Execution(cause)),
                    Err(denial) if producer_output => {
                        Err(RebaseVerificationReason::IndexedSelectionDenied(denial))
                    }
                    Err(_) => Ok(Self::Keep),
                }
            }
            Fact::RetiredOutputEntity { .. } => Ok(Self::Keep),
            read if reads_source(read) => Ok(Self::Keep),
            _ if producer_output => Err(RebaseVerificationReason::UnsupportedDecisionFact),
            _ => Ok(Self::Keep),
        }
    }

    pub(super) fn apply(self, fact: Fact) -> Fact {
        match (self, fact) {
            (Self::Keep | Self::KeepSuperseded, fact) => fact,
            (
                Self::Field { locator, revision },
                Fact::Field { entity_id, .. } | Fact::AbsentField { entity_id, .. },
            ) => Fact::SourceFieldRevision {
                entity_id,
                locator,
                native_revision: Some(revision),
            },
            (
                Self::Aspect(native_revision),
                Fact::OptionalAspectAbsent {
                    entity_id, locator, ..
                },
            ) => Fact::SourceAspectRevision {
                entity_id,
                aspect: locator.aspect().aspect_key().clone(),
                native_revision,
            },
            (
                Self::Aspect(native_revision),
                Fact::SourceAspectRevision {
                    entity_id, aspect, ..
                },
            ) => Fact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            },
            (
                Self::Adjacency(native_revision),
                Fact::SourceAdjacencyRevision {
                    relation_kind,
                    anchor,
                    direction,
                    comparison_work_limit,
                    endpoints,
                    ..
                },
            ) => Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                endpoints,
            },
            (Self::Replace(rebased), _) => rebased,
            _ => unreachable!("prepared fact resolution must match its original fact"),
        }
    }
}
