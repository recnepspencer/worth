use crate::data::aspect::{Aspect, AspectMask, AspectVersion};
use crate::data::handle::NodeId;
use crate::data::proof::PartitionScopeSet;

#[cfg(test)]
mod reference_construction;
mod retained_charge;

use super::binding::OutputCommitOrdinal;
use crate::data::graph::OutputCommitPublicationReceipt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ScopePrecision {
    ExactAspectScopes,
    ConservativeLegacyUnion,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProducedAspectChange {
    pub(crate) aspect: Aspect,
    pub(crate) previous_version: u64,
    pub(crate) committed_version: u64,
    pub(crate) changed_scopes: PartitionScopeSet,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct NonEmptyCanonicalAspectChangeSet(Vec<ProducedAspectChange>);

impl NonEmptyCanonicalAspectChangeSet {
    pub(crate) fn new(mut changes: Vec<ProducedAspectChange>) -> Option<Self> {
        changes.sort_by_key(|change| change.aspect.index());
        changes.dedup_by_key(|change| change.aspect.index());
        (!changes.is_empty()).then_some(Self(changes))
    }

    pub(crate) fn as_slice(&self) -> &[ProducedAspectChange] {
        &self.0
    }

    #[cfg(test)]
    pub(crate) fn first_mut_for_test(&mut self) -> &mut ProducedAspectChange {
        &mut self.0[0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProducedAspectDelta {
    pub(crate) producer: NodeId,
    pub(crate) output_commit_ordinal: OutputCommitOrdinal,
    pub(crate) committed_output_version: AspectVersion,
    pub(crate) changes: NonEmptyCanonicalAspectChangeSet,
    pub(crate) scope_precision: ScopePrecision,
}

#[derive(Debug)]
pub(crate) struct CommittedProducedAspectDelta {
    delta: ProducedAspectDelta,
    _performed: PerformedOutputCommit,
}

worth_proof::authority_marker!(OutputCommitAuthority);

struct PublishOutputCommit;
impl worth_proof::ActionMarker for PublishOutputCommit {}

type PerformedOutputCommit = worth_proof::Performed<PublishOutputCommit, OutputCommitAuthority>;

impl CommittedProducedAspectDelta {
    pub(crate) fn after_publication(
        delta: ProducedAspectDelta,
        _receipt: &OutputCommitPublicationReceipt,
    ) -> Self {
        Self {
            delta,
            _performed: worth_proof::Performed::record(&OutputCommitAuthority::witness(), ()),
        }
    }

    pub(crate) fn delta(&self) -> &ProducedAspectDelta {
        &self.delta
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticOutputCommitDecision {
    Unchanged {
        retained_version: AspectVersion,
    },
    Changed {
        committed_version: AspectVersion,
        changes: NonEmptyCanonicalAspectChangeSet,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputCommitContractViolation {
    DeclaredUnchangedWithSemanticChanges,
}

impl SemanticOutputCommitDecision {
    pub(crate) fn validate_declared_change(
        previous: AspectVersion,
        candidate: AspectVersion,
        produced_aspects: AspectMask,
        declared_unchanged: bool,
    ) -> Result<Self, OutputCommitContractViolation> {
        let changes = previous
            .slots()
            .iter()
            .zip(candidate.slots())
            .enumerate()
            .filter_map(|(index, (&before, &after))| {
                let aspect = Aspect::new(index as u8);
                (before != after && produced_aspects.contains(AspectMask::from_aspect(aspect)))
                    .then_some(ProducedAspectChange {
                        aspect,
                        previous_version: before,
                        committed_version: after,
                        changed_scopes: PartitionScopeSet::default(),
                    })
            })
            .collect::<Vec<_>>();
        let Some(changes) = NonEmptyCanonicalAspectChangeSet::new(changes) else {
            return Ok(Self::Unchanged {
                retained_version: previous,
            });
        };
        if declared_unchanged {
            return Err(OutputCommitContractViolation::DeclaredUnchangedWithSemanticChanges);
        }
        Ok(Self::Changed {
            committed_version: candidate,
            changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::output::ChangedRegion;

    #[test]
    fn contradictory_unchanged_output_is_denied_before_performed_authority_exists() {
        let previous = AspectVersion::from_updates([(Aspect::new(1), 1)]);
        let candidate = AspectVersion::from_updates([(Aspect::new(1), 2)]);

        assert_eq!(
            SemanticOutputCommitDecision::validate_declared_change(
                previous,
                candidate,
                AspectMask::ALL,
                true,
            ),
            Err(OutputCommitContractViolation::DeclaredUnchangedWithSemanticChanges)
        );
    }

    #[test]
    fn exact_aspect_scopes_and_legacy_multi_aspect_union_are_distinguishable() {
        let price = Aspect::new(1);
        let risk = Aspect::new(2);
        let previous = AspectVersion::zero();
        let candidate = AspectVersion::from_updates([(price, 1), (risk, 1)]);
        let exact = NonEmptyCanonicalAspectChangeSet::from_versions_and_regions(
            previous,
            candidate,
            &[
                (price, ChangedRegion::new("market")),
                (risk, ChangedRegion::new("risk")),
            ],
            &[],
        )
        .unwrap();
        assert_eq!(exact.1, ScopePrecision::ExactAspectScopes);
        assert_ne!(
            exact.0.as_slice()[0].changed_scopes,
            exact.0.as_slice()[1].changed_scopes
        );

        let legacy = NonEmptyCanonicalAspectChangeSet::from_versions_and_regions(
            previous,
            candidate,
            &[],
            &[ChangedRegion::new("legacy")],
        )
        .unwrap();
        assert_eq!(legacy.1, ScopePrecision::ConservativeLegacyUnion);
        assert_eq!(
            legacy.0.as_slice()[0].changed_scopes,
            legacy.0.as_slice()[1].changed_scopes
        );
    }
}
