//! What a group answer read on an unpublished candidate rests on (#361).
//!
//! A candidate has no group state of its own: commit finalization names a
//! scheme's next groups against the basis groups, from the commit's claims
//! and from every group that loses a member the commit ends. So a basis
//! answer is the candidate's answer exactly while the candidate claims no
//! partition of the scheme and leaves every member record the answer rests
//! on live: the member read, every member of the group holding it, and every
//! member of the one live group a held group name still designates. An
//! answer resting on name history (an unknown or ended member name, a held
//! group that ended, split, merged or is unknown) has no such records.

use std::collections::BTreeSet;

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameResolution,
    PersistentNameSelection, PersistentNameSubjectKind, PersistentNameTarget,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::{
    PersistentGroupAnswer, PersistentGroupRead, PersistentGroupSubject, PersistentGroupUnobservable,
};

/// The installed scheme a group read observes, and the live member records
/// its answer at the basis rests on (`None`: it rests on name history).
pub struct CandidateGroupSupport {
    pub scheme: String,
    pub members: Option<BTreeSet<EntityId>>,
}

impl PersistentGroupRead {
    /// Observes this read at `snapshot`, the candidate's commit basis, with
    /// the member records its answer rests on.
    ///
    /// Complexity: O(members * name lineage path).
    pub fn candidate_support(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<CandidateGroupSupport, PersistentGroupUnobservable> {
        let scheme = runtime
            .persistent_group_scheme_id(&self.group_kind)
            .ok_or(PersistentGroupUnobservable)?;
        let members = runtime
            .with_persistent_names_at_snapshot(snapshot, |view| {
                let answer = self.observe_in(&view, &scheme)?;
                Ok(self.members_in(&answer, &view))
            })
            .map_err(|_| PersistentGroupUnobservable)??;
        Ok(CandidateGroupSupport { scheme, members })
    }

    /// The member records `answer`, observed at `snapshot`, rests on.
    pub fn rested_members(
        &self,
        answer: &PersistentGroupAnswer,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<Option<BTreeSet<EntityId>>, PersistentGroupUnobservable> {
        runtime
            .with_persistent_names_at_snapshot(snapshot, |view| self.members_in(answer, &view))
            .map_err(|_| PersistentGroupUnobservable)
    }

    fn members_in(
        &self,
        answer: &PersistentGroupAnswer,
        view: &PersistentNameObservationView<'_>,
    ) -> Option<BTreeSet<EntityId>> {
        let mut names = Vec::new();
        match (self.subject, answer) {
            (PersistentGroupSubject::Member(member), PersistentGroupAnswer::Member(group)) => {
                names.push(member);
                if let Some(group) = group {
                    let key = view.recorded_name(PersistentNameSubjectKind::Group, group)?;
                    let members = view.group_members(key)?;
                    names.extend(members.iter().map(|member| *member.as_bytes()));
                }
            }
            (PersistentGroupSubject::Resolution(held), PersistentGroupAnswer::Live(groups)) => {
                let [(group, members)] = groups.as_slice() else {
                    return None;
                };
                if *group != held {
                    return None;
                }
                names.extend(members.iter().copied());
            }
            _ => return None,
        }
        names.iter().map(|name| live_holder(view, name)).collect()
    }
}

/// The live entity record a member name designates itself, or `None` when
/// the name is unknown or its record ended.
fn live_holder(view: &PersistentNameObservationView<'_>, name: &[u8; 32]) -> Option<EntityId> {
    let key = view.recorded_name(PersistentNameSubjectKind::Entity, name)?;
    let PersistentNameKey::Entity(identity) = key else {
        return None;
    };
    match view
        .resolve(key, &PersistentNameSelection::AllDescendants)
        .result
    {
        Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity {
            name, entity, ..
        })) if name == identity => Some(entity),
        _ => None,
    }
}
