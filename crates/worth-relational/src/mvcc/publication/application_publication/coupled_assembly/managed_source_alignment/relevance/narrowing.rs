//! Narrowing a relevant kind to what the probed scope's Source read (its
//! reach), and the one change no reach can narrow away: another scope coming
//! to answer to the probed scope's key.
use super::*;

impl SourceRelevance<'_> {
    /// Whether the Source read `entity`. Without a reach, or where a nested
    /// Parent relation or a projection route reads records the reach does
    /// not hold, every record of a relevant kind counts.
    pub(super) fn reads_entity(&self, entity: EntityId) -> bool {
        match &self.reach {
            Some(reach) if self.membership_is_the_whole_read() => {
                entity == self.probed || reach.entities.contains(&entity)
            }
            _ => true,
        }
    }

    /// Whether membership read an adjacency list the edge lies on. The
    /// reach holds only the lists membership read: where a projection route
    /// reads lists of its own, every edge of a relevant kind counts.
    pub(super) fn reads_edge(&self, kind: KindId, source: EntityId, target: EntityId) -> bool {
        match &self.reach {
            Some(reach) if self.program.projection_routes().is_empty() => {
                reach.anchors.contains(&(source, kind, true))
                    || reach.anchors.contains(&(target, kind, false))
            }
            _ => true,
        }
    }

    pub(super) fn reads_relation(
        &self,
        relation: crate::identity::data::RelationId,
        kind: KindId,
    ) -> bool {
        match self.edges.get(&relation) {
            Some(&(source, target)) => self.reads_edge(kind, source, target),
            None => true,
        }
    }

    fn membership_is_the_whole_read(&self) -> bool {
        self.footprint.relations.is_empty() && self.program.projection_routes().is_empty()
    }

    /// Whether this change makes a scope other than the probed one answer to
    /// the probed scope's key: created or restored with it, or its key field
    /// rewritten to it. An opaque or unreadable key counts. This holds on
    /// any scope, read by the probed Source or not, since the key would then
    /// no longer name the probed scope alone.
    pub(super) fn takes_probed_key(&self, delta: &CanonicalRecordAspectDelta, id: EntityId) -> bool {
        let Some((kind, aspect, field)) = &self.footprint.scope_key else {
            return false;
        };
        if delta.kind_id != *kind || id == self.probed {
            return false;
        }
        let written = match delta.structural_change {
            Change::Created | Change::Rematerialized => true,
            Change::Updated => field_changed(delta, aspect, field),
            _ => false,
        };
        written && (delta.contains_opaque_aspect || self.answers_to_probed_key(id))
    }

    /// A changed scope Native described with no aspect delta: nothing says
    /// its key was left alone, so its current key decides.
    pub(super) fn may_hold_probed_key(&self, id: EntityId, kind: KindId) -> bool {
        id != self.probed
            && self
                .footprint
                .scope_key
                .as_ref()
                .is_some_and(|(scope, _, _)| *scope == kind)
            && self.answers_to_probed_key(id)
    }

    fn answers_to_probed_key(&self, id: EntityId) -> bool {
        match (&self.probed_key, (self.key_of)(id)) {
            (Some(probed), Some(sibling)) => *probed == sibling,
            _ => true,
        }
    }
}
