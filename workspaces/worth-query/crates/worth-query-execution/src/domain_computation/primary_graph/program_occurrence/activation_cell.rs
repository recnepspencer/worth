use std::sync::{Arc, OnceLock};

use worth_relational::facade::identity::EntityId;

/// The identity of this application's one branch program activation record.
///
/// Bootstrap publishes the identity exactly once, in the same installation that
/// lowered the invariant adapters and the runtime sharing this cell, so every
/// reader resolves the same record and no lane can rebind it afterwards. A cell
/// that was never published is not a permissive default: readers that find it
/// empty refuse the work they were about to do.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramActivationCell {
    identity: Arc<OnceLock<EntityId>>,
}

impl WorthQueryProgramActivationCell {
    pub(in crate::domain_computation::primary_graph) fn unpublished() -> Self {
        Self {
            identity: Arc::new(OnceLock::new()),
        }
    }

    /// Publishes the activation record's identity, naming the identity already
    /// carried when one is present.
    pub(in crate::domain_computation::primary_graph) fn publish(
        &self,
        identity: EntityId,
    ) -> Result<(), EntityId> {
        match self.identity.set(identity) {
            Ok(()) => Ok(()),
            Err(rejected) => Err(self.identity.get().copied().unwrap_or(rejected)),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn published(&self) -> Option<EntityId> {
        self.identity.get().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryProgramActivationCell;
    use worth_relational::facade::identity::{EntityId, PartitionId};

    fn activation_record(slot: u64) -> EntityId {
        EntityId::new(PartitionId::main(), slot, 1)
    }

    #[test]
    fn an_unpublished_cell_names_no_activation_record() {
        assert_eq!(
            WorthQueryProgramActivationCell::unpublished().published(),
            None
        );
    }

    #[test]
    fn the_first_published_identity_is_the_only_one_the_cell_ever_carries() {
        let cell = WorthQueryProgramActivationCell::unpublished();
        cell.publish(activation_record(7))
            .expect("the first publication binds the activation record");
        assert_eq!(
            cell.publish(activation_record(9)),
            Err(activation_record(7)),
            "a second publication must name the identity already bound"
        );
        assert_eq!(cell.published(), Some(activation_record(7)));
    }

    #[test]
    fn every_clone_of_one_cell_reads_the_same_activation_record() {
        let cell = WorthQueryProgramActivationCell::unpublished();
        let reader = cell.clone();
        assert_eq!(reader.published(), None);
        cell.publish(activation_record(4))
            .expect("the first publication binds the activation record");
        assert_eq!(reader.published(), Some(activation_record(4)));
    }
}
