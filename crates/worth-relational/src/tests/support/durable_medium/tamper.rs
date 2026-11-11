//! Damage the test medium's durable contents, standing in for storage that
//! returns bytes Relational did not write.

use super::TestDurableMedium;
use crate::durability::data::RelationalNativeCheckpoint;

impl TestDurableMedium {
    /// Flip every bit of one byte of the filed checkpoint, keeping the
    /// position it is filed under.
    pub(crate) fn flip_checkpoint_byte(&self, offset: usize) {
        let mut journal = self.journal();
        let filed = journal
            .checkpoint
            .take()
            .expect("medium holds a checkpoint");
        let mut bytes = filed.checkpoint().bytes().to_vec();
        bytes[offset] ^= 0xff;
        journal.checkpoint = Some(
            crate::durability::medium::RelationalDurableCheckpointSource::new(
                filed.covered(),
                filed.covered_transitions(),
                RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
            ),
        );
    }

    /// Damage the filed checkpoint's payload so it no longer decodes: the
    /// first byte after the frame magic is the payload's map marker.
    pub(crate) fn damage_checkpoint_payload(&self) {
        self.flip_checkpoint_byte(
            crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC.len(),
        );
    }
}
