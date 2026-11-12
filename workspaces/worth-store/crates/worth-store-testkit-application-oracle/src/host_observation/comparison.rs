//! Semantic positions establish correspondence; issuer numbers never order pairs.
use super::projection::{HeadProjection, StateProjection};
use crate::facade::{
    DurableIdentityCorrespondence, IdentityDivergence, ObservedDurableIdentity as Id,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionPart {
    GraphCardinality,
    EntityMeaning,
    RelationMeaning,
    LineagePresence,
    LifetimePresence,
    Head,
    OccurrenceCardinality,
    OccurrenceParent,
    OccurrenceProvenance,
    OccurrenceComponent,
    ReferenceHistory,
    CustodyCardinality,
    CustodyState,
    Receipt,
    NativeSemantic,
    Outcome,
    FateEvidence,
    ProgramBinding,
    MigrationSteps,
    BranchRoster,
    Idempotency,
}
#[derive(Debug)]
pub enum HostDivergence {
    Identity(IdentityDivergence),
    Meaning { part: ProjectionPart, index: usize },
}
impl From<IdentityDivergence> for HostDivergence {
    fn from(value: IdentityDivergence) -> Self {
        Self::Identity(value)
    }
}
#[derive(Default)]
pub struct HostOracleComparison {
    identities: DurableIdentityCorrespondence,
    reference_history: super::reference_history::ReferenceOccurrencePrefix,
}
impl HostOracleComparison {
    pub fn identities_mut(&mut self) -> &mut DurableIdentityCorrespondence {
        &mut self.identities
    }
    pub fn reference_occurrences(
        &self,
        head: &HeadProjection,
    ) -> Result<Vec<super::projection::OccurrenceProjection>, HostDivergence> {
        self.reference_history.trace(head)
    }
}
pub fn compare_head(
    map: &mut DurableIdentityCorrespondence,
    home: &HeadProjection,
    reference: &HeadProjection,
) -> Result<(), HostDivergence> {
    if home.branch != reference.branch || home.generation != reference.generation {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::Head,
            index: 0,
        });
    }
    map.admit(Id::WorldOwner(home.owner), Id::WorldOwner(reference.owner))?;
    map.admit(
        Id::WorldBranch {
            owner: home.owner,
            name: home.branch.clone(),
        },
        Id::WorldBranch {
            owner: reference.owner,
            name: reference.branch.clone(),
        },
    )?;
    map.admit(
        Id::WorldIncarnation {
            owner: home.owner,
            ordinal: home.incarnation,
        },
        Id::WorldIncarnation {
            owner: reference.owner,
            ordinal: reference.incarnation,
        },
    )?;
    map.admit(
        Id::WorldCommit {
            owner: home.owner,
            ordinal: home.commit,
        },
        Id::WorldCommit {
            owner: reference.owner,
            ordinal: reference.commit,
        },
    )?;
    Ok(())
}
pub fn compare(
    comparison: &mut HostOracleComparison,
    home: &StateProjection,
    reference: &StateProjection,
) -> Result<(), HostDivergence> {
    comparison
        .reference_history
        .observe(&reference.occurrences)?;
    let expected_history = comparison.reference_history.trace(&reference.head)?;
    let map = &mut comparison.identities;
    if home.entities.len() != 3
        || reference.entities.len() != 3
        || home.relations.len() != reference.relations.len()
    {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::GraphCardinality,
            index: 0,
        });
    }
    for (index, (h, r)) in home.entities.iter().zip(&reference.entities).enumerate() {
        if h.kind_name != r.kind_name
            || h.lifecycle != r.lifecycle
            || h.authoritative_aspect_state != r.authoritative_aspect_state
        {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::EntityMeaning,
                index,
            });
        }
        map.admit(Id::Entity(h.entity_id), Id::Entity(r.entity_id))?;
        map.admit(
            Id::NativeVersion(h.created_at_version),
            Id::NativeVersion(r.created_at_version),
        )?;
        match (h.lineage_id, r.lineage_id) {
            (None, None) => (),
            (Some(h), Some(r)) => map.admit(Id::Lineage(h), Id::Lineage(r))?,
            (None, Some(_)) | (Some(_), None) => {
                return Err(HostDivergence::Meaning {
                    part: ProjectionPart::LineagePresence,
                    index,
                })
            }
        }
        match (h.retired_at_version, r.retired_at_version) {
            (None, None) => (),
            (Some(h), Some(r)) => map.admit(Id::NativeVersion(h), Id::NativeVersion(r))?,
            (None, Some(_)) | (Some(_), None) => {
                return Err(HostDivergence::Meaning {
                    part: ProjectionPart::LifetimePresence,
                    index,
                })
            }
        }
    }
    for (index, (h, r)) in home.relations.iter().zip(&reference.relations).enumerate() {
        if h.kind_name != r.kind_name
            || h.lifecycle != r.lifecycle
            || h.authoritative_aspect_state != r.authoritative_aspect_state
        {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::RelationMeaning,
                index,
            });
        }
        map.admit(Id::Relation(h.relation_id), Id::Relation(r.relation_id))?;
        map.admit(Id::Entity(h.source), Id::Entity(r.source))?;
        map.admit(Id::Entity(h.target), Id::Entity(r.target))?;
        map.admit(
            Id::NativeVersion(h.created_at_version),
            Id::NativeVersion(r.created_at_version),
        )?;
        match (h.retired_at_version, r.retired_at_version) {
            (None, None) => (),
            (Some(h), Some(r)) => map.admit(Id::NativeVersion(h), Id::NativeVersion(r))?,
            (None, Some(_)) | (Some(_), None) => {
                return Err(HostDivergence::Meaning {
                    part: ProjectionPart::LifetimePresence,
                    index,
                })
            }
        }
    }
    super::idempotency::compare(map, home, reference)?;
    compare_head(map, &home.head, &reference.head)?;
    // Home must expose its actual durable prefix. Memory's ordinary cache may
    // retire descriptions that the oracle already observed in earlier prefixes.
    if !home.history_complete || home.history_crossed_retired {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::OccurrenceCardinality,
            index: 0,
        });
    }
    super::occurrence::compare(map, &home.occurrences, &expected_history)?;
    super::custody::compare(map, &home.custody, &reference.custody)?;
    Ok(())
}
