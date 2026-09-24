use super::{UiIntentPostureCommit, UiIntentPostureObservation};

pub(crate) struct UiIntentPostureTable {
    next_owner_order: u64,
    committed_postures: usize,
}

impl UiIntentPostureTable {
    pub(crate) const fn new() -> Self {
        Self {
            next_owner_order: 1,
            committed_postures: 0,
        }
    }

    pub(crate) const fn prepare(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
        reference: crate::fact_contract::UiIntentPostureReference,
        posture: crate::fact_contract::UiIntentPostureKind,
    ) -> Option<(UiIntentPostureObservation, UiIntentPostureCommit)> {
        if self.next_owner_order == u64::MAX {
            return None;
        }
        Some(UiIntentPostureObservation::new(
            graph_node,
            target,
            reference,
            posture,
            self.next_owner_order,
        ))
    }

    /// Owner order is this table's publication sequence, and it advances
    /// only on commit. Every posture prepared before the previous one
    /// commits, such as one per transition in a batched ingress or one
    /// queued behind a publication in flight, was stamped with the same next
    /// order. A deferred posture therefore takes its order when it is
    /// admitted for publication, not when it was prepared.
    pub(crate) fn bind_publication_order(
        &self,
        observation: UiIntentPostureObservation,
        commit: UiIntentPostureCommit,
    ) -> Option<(UiIntentPostureObservation, UiIntentPostureCommit)> {
        let _ = commit;
        if self.next_owner_order == u64::MAX {
            return None;
        }
        Some(observation.with_owner_order(self.next_owner_order))
    }

    pub(crate) fn commit(&mut self, commit: UiIntentPostureCommit) {
        assert_eq!(commit.owner_order(), self.next_owner_order);
        self.next_owner_order += 1;
        self.committed_postures = self
            .committed_postures
            .checked_add(1)
            .expect("bounded intent posture count exhausted");
    }
}
