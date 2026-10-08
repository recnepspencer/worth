use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_relational::facade::{identity::VersionId, runtime::RelationalFieldRevision};

use crate::domain_computation::primary_graph::application_attempt::reobserve_indexed_entity_selection;

use super::{
    adjacency, retirement, RebaseVerificationReason, WorthQueryApplicationObservedFact as Fact,
};

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
    pub(super) fn prepare(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        fact: &Fact,
        retired: &std::collections::BTreeSet<worth_relational::facade::identity::EntityId>,
        producer_output: bool,
        maximum_pair_rebase_work: usize,
        indexed_rebase_work: &mut usize,
        prepared_endpoints: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
        >,
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
                    prepared_endpoints,
                ) {
                    Some(rebased) => Ok(Self::Replace(rebased)),
                    None if producer_output => Err(unavailable),
                    None => Ok(Self::Keep),
                }
            }
            Fact::IndexedEntitySelection { .. } => {
                // The selection is observed again at the committed snapshot,
                // so it names the members this commit itself published.
                match reobserve_indexed_entity_selection(
                    fact,
                    runtime,
                    snapshot,
                    indexed_rebase_work,
                ) {
                    Ok(observed) => Ok(Self::Replace(observed)),
                    Err(denial) if producer_output => {
                        Err(RebaseVerificationReason::IndexedSelectionDenied(denial))
                    }
                    Err(_) => Ok(Self::Keep),
                }
            }
            Fact::RetiredOutputEntity { .. } => Ok(Self::Keep),
            Fact::SourceEntity { .. }
            | Fact::Entity { .. }
            | Fact::SourceFieldRevision {
                native_revision: Some(_),
                ..
            } if producer_output => match fact.source_currentness_in(runtime, snapshot, 1) {
                Ok((true, _)) => Ok(Self::Keep),
                Ok((false, _)) => Ok(Self::KeepSuperseded),
                Err(_) => Err(unavailable),
            },
            Fact::SourceEntity { .. }
            | Fact::Entity { .. }
            | Fact::SourceFieldRevision {
                native_revision: Some(_),
                ..
            } => Ok(Self::Keep),
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
