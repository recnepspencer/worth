use std::sync::Arc;

#[derive(Clone)]
#[doc(hidden)]
pub struct WorkflowTransitionReplayProjection {
    pub(in crate::domain_computation::primary_graph) identity: String,
    pub(in crate::domain_computation::primary_graph) identity_bytes: [u8; 32],
    pub(in crate::domain_computation::primary_graph) node_path: String,
    pub(in crate::domain_computation::primary_graph) terminal: bool,
    pub(in crate::domain_computation::primary_graph) navigation_back: bool,
    pub(in crate::domain_computation::primary_graph) operation_receipt_identity: Option<[u8; 32]>,
}

impl WorkflowTransitionReplayProjection {
    fn retained_dynamic_charge_bytes(&self) -> usize {
        self.identity
            .capacity()
            .saturating_add(self.node_path.capacity())
    }
}

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorkflowTransitionReplayRetention {
    tail: Option<Arc<RetainedWorkflowTransitionReplay>>,
    len: usize,
    retained_charge_bytes: usize,
}

struct RetainedWorkflowTransitionReplay {
    replay: WorkflowTransitionReplayProjection,
    previous: Option<Arc<Self>>,
}

impl Drop for RetainedWorkflowTransitionReplay {
    fn drop(&mut self) {
        // A settled history is an Arc-linked prefix. Letting its last owner
        // recurse through `previous` would consume one stack frame per replay.
        let mut previous = self.previous.take();
        while let Some(link) = previous {
            match Arc::try_unwrap(link) {
                Ok(mut replay) => previous = replay.previous.take(),
                Err(_) => break, // A shared prefix belongs to another owner.
            }
        }
    }
}

impl WorkflowTransitionReplayRetention {
    pub(in crate::domain_computation::primary_graph) fn from_replays(
        replays: impl IntoIterator<Item = WorkflowTransitionReplayProjection>,
    ) -> Self {
        replays
            .into_iter()
            .fold(Self::default(), |retained, replay| retained.append(replay))
    }

    pub(super) fn append(&self, replay: WorkflowTransitionReplayProjection) -> Self {
        Self {
            retained_charge_bytes: self
                .retained_charge_bytes
                .saturating_add(std::mem::size_of::<RetainedWorkflowTransitionReplay>())
                .saturating_add(replay.retained_dynamic_charge_bytes()),
            tail: Some(Arc::new(RetainedWorkflowTransitionReplay {
                replay,
                previous: self.tail.clone(),
            })),
            len: self.len.saturating_add(1),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn materialize(
        &self,
    ) -> Box<[WorkflowTransitionReplayProjection]> {
        let mut replays = Vec::with_capacity(self.len);
        let mut current = self.tail.as_deref();
        while let Some(retained) = current {
            replays.push(retained.replay.clone());
            current = retained.previous.as_deref();
        }
        replays.reverse();
        replays.into_boxed_slice()
    }

    pub(super) const fn retained_charge_bytes(&self) -> usize {
        self.retained_charge_bytes
    }

    #[cfg(test)]
    pub(super) const fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replay(identity: u8) -> WorkflowTransitionReplayProjection {
        WorkflowTransitionReplayProjection {
            identity: format!("transition-{identity}"),
            identity_bytes: [identity; 32],
            node_path: format!("node-{identity}"),
            terminal: false,
            navigation_back: false,
            operation_receipt_identity: None,
        }
    }

    #[test]
    fn append_shares_the_retained_prefix_and_materializes_in_authored_order() {
        let retained = WorkflowTransitionReplayRetention::from_replays([replay(1), replay(2)]);
        let advanced = retained.append(replay(3));

        assert_eq!(retained.len(), 2);
        assert_eq!(advanced.len(), 3);
        assert!(Arc::ptr_eq(
            retained.tail.as_ref().expect("retained tail exists"),
            advanced
                .tail
                .as_ref()
                .and_then(|tail| tail.previous.as_ref())
                .expect("advanced replay shares its prefix"),
        ));
        let materialized = advanced.materialize();
        let identities = materialized
            .iter()
            .map(|replay| replay.identity.as_str())
            .collect::<Vec<_>>();
        assert_eq!(identities, ["transition-1", "transition-2", "transition-3"]);
    }

    #[test]
    fn dropping_a_long_history_and_shared_prefix_uses_bounded_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let prefix = WorkflowTransitionReplayRetention::from_replays(
                    (0..10_000).map(|index| replay(index as u8)),
                );
                let advanced = prefix.append(replay(7));
                drop(advanced);
                let materialized = prefix.materialize();
                assert_eq!(materialized.len(), 10_000);
                assert_eq!(materialized[0].identity, "transition-0");
                assert_eq!(materialized[9_999].identity, "transition-15");
                drop(prefix);

                let prefix = WorkflowTransitionReplayRetention::from_replays(
                    (0..10_000).map(|index| replay(index as u8)),
                );
                let advanced = prefix.append(replay(7));
                assert_eq!(advanced.len(), 10_001);
                drop(prefix);
                drop(advanced);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
