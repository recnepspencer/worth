//! Compare each World occurrence with the exact owner movement it records.
use super::{
    comparison::{HostDivergence, ProjectionPart},
    projection::{OccurrenceProjection, OccurrenceProvenance},
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignalMovement {
    Advanced,
    Forked,
}
impl SignalMovement {
    pub fn observe(
        value: worth_query_host::facade::primary_graph::WorthQueryOracleSignalMovementForTest,
    ) -> Self {
        use worth_query_host::facade::primary_graph::WorthQueryOracleSignalMovementForTest as S;
        match value {
            S::Advanced => Self::Advanced,
            S::Forked => Self::Forked,
        }
    }
}
pub fn changed(value: worth_runtime_world::facade::CompositeComponentChangePosture) -> bool {
    use worth_runtime_world::facade::CompositeComponentChangePosture as C;
    match value {
        C::RetainExact => false,
        C::Published => true,
    }
}
pub fn compare(
    map: &mut DurableIdentityCorrespondence,
    home: &[OccurrenceProjection],
    reference: &[OccurrenceProjection],
) -> Result<(), HostDivergence> {
    if home.len() != reference.len() {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::OccurrenceCardinality,
            index: 0,
        });
    }
    for (index, (h, r)) in home.iter().zip(reference).enumerate() {
        compare_components(map, h, r, index)?;
        map.admit(Id::WorldOwner(h.owner), Id::WorldOwner(r.owner))?;
        map.admit(
            Id::WorldCommit {
                owner: h.owner,
                ordinal: h.commit,
            },
            Id::WorldCommit {
                owner: r.owner,
                ordinal: r.commit,
            },
        )?;
        match (h.parent, r.parent) {
            (None, None) => (),
            (Some(hp), Some(rp)) => map.admit(
                Id::WorldCommit {
                    owner: h.owner,
                    ordinal: hp,
                },
                Id::WorldCommit {
                    owner: r.owner,
                    ordinal: rp,
                },
            )?,
            (None, Some(_)) | (Some(_), None) => {
                return Err(HostDivergence::Meaning {
                    part: ProjectionPart::OccurrenceParent,
                    index,
                })
            }
        }
        match (&h.provenance, &r.provenance) {
            (OccurrenceProvenance::Bootstrap(hp), OccurrenceProvenance::Bootstrap(rp)) => map
                .admit(
                    Id::WorldBootstrap {
                        owner: h.owner,
                        ordinal: *hp,
                    },
                    Id::WorldBootstrap {
                        owner: r.owner,
                        ordinal: *rp,
                    },
                )?,
            (OccurrenceProvenance::Publication(hp), OccurrenceProvenance::Publication(rp)) => map
                .admit(
                Id::WorldAttempt {
                    owner: h.owner,
                    ordinal: *hp,
                },
                Id::WorldAttempt {
                    owner: r.owner,
                    ordinal: *rp,
                },
            )?,
            (OccurrenceProvenance::Bootstrap(_), OccurrenceProvenance::Publication(_))
            | (OccurrenceProvenance::Publication(_), OccurrenceProvenance::Bootstrap(_)) => {
                return Err(HostDivergence::Meaning {
                    part: ProjectionPart::OccurrenceProvenance,
                    index,
                })
            }
        }
    }
    Ok(())
}
fn compare_components(
    map: &mut DurableIdentityCorrespondence,
    h: &OccurrenceProjection,
    r: &OccurrenceProjection,
    index: usize,
) -> Result<(), HostDivergence> {
    let bad = || HostDivergence::Meaning {
        part: ProjectionPart::OccurrenceComponent,
        index,
    };
    if h.relational_changed != r.relational_changed
        || h.signal_changed != r.signal_changed
        || h.signal_movement != r.signal_movement
    {
        return Err(bad());
    }
    match (&h.relational_publication, &r.relational_publication) {
        (None, None) => (),
        (Some(h), Some(r)) => {
            map.admit(
                Id::NativeCommit(h.commit_id()),
                Id::NativeCommit(r.commit_id()),
            )?;
            map.admit(
                Id::NativeVersion(h.version_id()),
                Id::NativeVersion(r.version_id()),
            )?;
            map.admit(
                Id::NativeBranch(h.authoring_branch().clone()),
                Id::NativeBranch(r.authoring_branch().clone()),
            )?;
        }
        (None, Some(_)) | (Some(_), None) => return Err(bad()),
    }
    match (&h.relational_fork, &r.relational_fork) {
        (None, None) => (),
        (Some(h), Some(r)) => {
            map.admit(Id::NativeBranch(h.clone()), Id::NativeBranch(r.clone()))?
        }
        (None, Some(_)) | (Some(_), None) => return Err(bad()),
    }
    Ok(())
}
