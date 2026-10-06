use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_relational::facade::{
    identity::VersionId, mvcc::CompanionPreflightStop, runtime::RelationalFieldRevision,
};

use crate::domain_computation::primary_graph::application_attempt::{
    reobserve_indexed_entity_selection, Movement,
};

use super::{
    adjacency, retirement, RebaseVerificationReason, WorthQueryApplicationObservedFact as Fact,
};

/// A fact a source query read. It stays at the revision it was read at, so
/// it is the one kind the committed effect can leave stale: every other fact
/// is observed again at the committed snapshot.
pub(super) fn reads_source(fact: &Fact) -> bool {
    matches!(
        fact,
        Fact::SourceEntity { .. } | Fact::Entity { .. } | Fact::SourceFieldRevision { .. }
    )
}

/// The most resolving `fact` can spend: one unit, or for a producer's source
/// read the most its comparison can cost.
pub(super) fn reserved_work(fact: &Fact, producer_output: bool) -> usize {
    if producer_output && reads_source(fact) {
        fact.most_source_comparison_work()
    } else {
        1
    }
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
    /// `spent` enters as the fact's reserved work and leaves as the work it
    /// spent, which only a source read's comparison makes smaller.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        retired: &std::collections::BTreeSet<worth_relational::facade::identity::EntityId>,
        producer_output: bool,
        maximum_pair_rebase_work: usize,
        indexed_rebase_work: &mut usize,
        spent: &mut usize,
    ) -> Result<Self, RebaseVerificationReason> {
        let unavailable = RebaseVerificationReason::NativeRevisionUnavailable;
        if let Some(retirement) = retirement::resolve(runtime, snapshot, fact, retired) {
            // No other fact may stand in for a retirement the snapshot lacks.
            return retirement.map(Self::Replace).ok_or(unavailable);
        }
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
            Fact::SourceFieldRevision {
                native_revision: None,
                ..
            } if producer_output => Err(unavailable),
            Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                comparison_work_limit,
                ..
            } => match runtime
                .read_truth()
                .project_snapshot(snapshot)
                .and_then(|view| {
                    view.bounded_adjacency_structural_revision(
                        *anchor,
                        *relation_kind,
                        *direction,
                        (*comparison_work_limit).min(maximum_pair_rebase_work),
                    )
                    .ok()
                }) {
                Some(comparison) => Ok(Self::Adjacency(comparison.revision())),
                None if producer_output => Err(unavailable),
                None => Ok(Self::Keep),
            },
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
            Fact::IndexedEntitySelection { .. } => {
                // The selection is observed again at the committed snapshot,
                // so it names the members this commit itself published. It is
                // charged what it examined, as the decision that read it was.
                // Facts are walked in key order, not the order they were read
                // in, so the width that decision admitted bounds each
                // observation and no selection waits on the ones before it.
                // One the width cannot pay for is not decided, and never kept.
                match reobserve_indexed_entity_selection(fact, runtime, snapshot) {
                    Some((observed, examined)) => {
                        let charged = examined.checked_add(1).ok_or(unpaid(None))?;
                        *indexed_rebase_work = indexed_rebase_work
                            .checked_sub(charged)
                            .ok_or_else(|| unpaid(Some((charged, *indexed_rebase_work))))?;
                        Ok(Self::Replace(observed))
                    }
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::RetiredOutputEntity { .. } => Ok(Self::Keep),
            read if reads_source(read) && producer_output => {
                // Granted the most the comparison can cost, so a failure is
                // one it could not answer, never one it could not pay for.
                match read.source_currentness_in(runtime, snapshot, *spent) {
                    Ok((movement, work)) => {
                        *spent = work;
                        Ok(match movement.movement() {
                            Movement::Unmoved => Self::Keep,
                            Movement::Moved => Self::KeepSuperseded,
                        })
                    }
                    Err(_) => Err(unavailable),
                }
            }
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

/// The rebase's indexed width could not pay `required` from `maximum`, or the
/// count overflowed.
fn unpaid(work: Option<(usize, usize)>) -> RebaseVerificationReason {
    let stop = work
        .and_then(|(required, maximum)| {
            Some(CompanionPreflightStop::WorkExhausted {
                required: u64::try_from(required).ok()?,
                maximum: u64::try_from(maximum).ok()?,
            })
        })
        .unwrap_or(CompanionPreflightStop::WorkCounterOverflow);
    RebaseVerificationReason::AdmissionDenied(stop)
}
