use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_relational::facade::{
    identity::VersionId,
    mvcc::CompanionPreflightStop,
    runtime::{AdjacencyStructuralRevisionDenial, RelationalFieldRevision},
};

use crate::domain_computation::primary_graph::application_attempt::{
    reobserve_indexed_entity_selection, IndexedReobservation, Movement,
    WorthQuerySourceCurrentnessFailure,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

use super::{
    adjacency, retirement, RebaseVerificationReason, WorthQueryApplicationObservedFact as Fact,
};

/// A fact a source query read. It stays at the revision it was read at, so
/// it is the one kind the walk asks the committed effect about: every other
/// fact is observed again at the committed snapshot. A field an owner call
/// read and the effect replaced is marked moved before the walk.
pub(super) fn reads_source(fact: &Fact) -> bool {
    matches!(
        fact,
        Fact::SourceEntity { .. } | Fact::Entity { .. } | Fact::SourceFieldRevision { .. }
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
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        retired: &std::collections::BTreeSet<worth_relational::facade::identity::EntityId>,
        producer_output: bool,
        admission: &mut InvalidationEditAdmission,
        indexed_width: &mut InvalidationEditAdmission,
    ) -> Result<Result<Self, RebaseVerificationReason>, CompanionPreflightStop> {
        let unavailable = RebaseVerificationReason::NativeRevisionUnavailable;
        if let Some(entity_id) = retirement::retired_anchor(fact, retired) {
            admission.charge_external_work(1)?;
            // No other fact may stand in for a retirement the snapshot lacks.
            return Ok(
                retirement::committed_retirement(runtime, snapshot, entity_id, fact)
                    .map(Self::Replace)
                    .ok_or(unavailable),
            );
        }
        if producer_output && reads_source(fact) {
            return Ok(
                match fact.source_currentness_in(runtime, snapshot, admission)? {
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
        admission.charge_external_work(1)?;
        let maximum_pair_rebase_work = admission.remaining_work();
        if matches!(
            fact,
            Fact::Relation { .. }
                | Fact::Adjacency { .. }
                | Fact::SourceAdjacencyRevision { .. }
                | Fact::IndexedEntitySelection { .. }
        ) {
            // The selected native adjacency revision and the indexed
            // selection each perform one indexed probe. A probe's per-call
            // cap alone does not spend request work.
            admission.charge_external_work(1)?;
        }
        Ok(Self::resolve(
            runtime,
            snapshot,
            fact,
            producer_output,
            maximum_pair_rebase_work,
            indexed_width,
        ))
    }

    fn resolve(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        producer_output: bool,
        maximum_pair_rebase_work: usize,
        indexed_width: &mut InvalidationEditAdmission,
    ) -> Result<Self, RebaseVerificationReason> {
        let unavailable = RebaseVerificationReason::NativeRevisionUnavailable;
        match fact {
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
                match runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_field_revision(*entity_id, &locator))
                {
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
                    *anchor,
                    *relation_kind,
                    *direction,
                    (*comparison_work_limit).min(maximum_pair_rebase_work),
                ) {
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
                    runtime,
                    snapshot,
                    fact,
                    maximum_pair_rebase_work,
                ) {
                    Some(rebased) => Ok(Self::Replace(rebased)),
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::IndexedEntitySelection {
                candidate_limit, ..
            } => {
                // The selection is observed again at the committed snapshot,
                // so it names the members this commit itself published. It is
                // charged what it examined, as the decision that read it was.
                // Facts are walked in key order, not the order they were read
                // in, so the width that decision admitted bounds each
                // observation and no selection waits on the ones before it.
                // The observation reserves the least of its worst case and
                // the width left before it reads, and reads within that
                // reservation. One the width cannot pay for is not decided,
                // and never kept.
                let most = candidate_limit.saturating_add(1);
                let reserved_work = most.min(indexed_width.remaining_work());
                let reserved = indexed_width
                    .reserve_external_work(u64::try_from(reserved_work).unwrap_or(u64::MAX))
                    .map_err(RebaseVerificationReason::AdmissionDenied)?;
                match reobserve_indexed_entity_selection(fact, runtime, snapshot, reserved_work) {
                    Ok((observed, examined)) => {
                        reserved
                            .settle(u64::try_from(examined.saturating_add(1)).unwrap_or(u64::MAX))
                            .map_err(RebaseVerificationReason::AdmissionDenied)?;
                        Ok(Self::Replace(observed))
                    }
                    Err(IndexedReobservation::Unpaid) => {
                        let maximum = indexed_width.charged_work();
                        Err(RebaseVerificationReason::AdmissionDenied(
                            CompanionPreflightStop::WorkExhausted {
                                required: maximum.saturating_add(1),
                                maximum,
                            },
                        ))
                    }
                    Err(IndexedReobservation::Unavailable) if producer_output => Err(unavailable),
                    Err(IndexedReobservation::Unavailable) => Ok(Self::Keep),
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
