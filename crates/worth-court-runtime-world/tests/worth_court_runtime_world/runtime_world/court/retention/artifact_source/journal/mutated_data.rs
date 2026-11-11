//! Adversarial DATA cuts retain actual published frames, never publication authority.
use super::*;
impl Recorder {
    pub(in super::super::super) fn untrusted_snapshot(
        &self,
        change: impl FnOnce(&mut Vec<(WorldDurableJournal, u64, Vec<u8>)>),
    ) -> Box<dyn WorldCheckpointReadSource> {
        let mut data = self
            .rows
            .lock()
            .unwrap()
            .iter()
            .map(|row| (row.journal, row.position, row.bytes.clone()))
            .collect();
        change(&mut data);
        Box::new(JournalSource {
            selected: None,
            rows: data
                .into_iter()
                .map(|(journal, position, bytes)| Stored {
                    journal,
                    position,
                    digest: Sha256::digest(&bytes).into(),
                    bytes,
                })
                .collect(),
        })
    }
}
