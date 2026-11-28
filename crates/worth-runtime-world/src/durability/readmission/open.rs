//! Recovered opens keep their exact decoded allocation through custody escape.
use super::{AdmittedWorldDisposition, RecoveredCustodyDisposition, WorldCheckpointImage};
use crate::durability::{
    DecodedWorldRecord, DurableWorldHead, WorldOpenAttemptRecord, WorldRecord,
};
use std::sync::Arc;

/// Reclaim custody for one persisted open, interpreted against its durable outcomes.
pub(crate) enum ExpectedReclaim<'a> {
    Published,
    Released(&'a AdmittedWorldDisposition),
    Pending(&'a DurableWorldHead),
}

impl WorldCheckpointImage {
    pub(crate) fn reclaim_expected<'a>(
        &'a self,
        open: &'a AdmittedWorldOpenAttempt,
    ) -> ExpectedReclaim<'a> {
        if self
            .history
            .get(&open.commit)
            .is_some_and(|entry| entry.represents_open(open.open()))
        {
            return ExpectedReclaim::Published;
        }
        match self.dispositions.get(&open.association.attempt()) {
            Some(release)
                if matches!(
                    release.disposition(),
                    RecoveredCustodyDisposition::Released(_)
                ) =>
            {
                ExpectedReclaim::Released(release)
            }
            _ => ExpectedReclaim::Pending(&open.expected),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldOpenAttempt {
    meaning: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldOpenAttempt {
    pub(super) fn from_validated_journal(
        meaning: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        assert!(matches!(&meaning.record, WorldRecord::OpenAttempt(_)));
        Self {
            meaning,
            position,
            digest,
        }
    }
    pub(crate) fn open(&self) -> &WorldOpenAttemptRecord {
        let WorldRecord::OpenAttempt(open) = &self.meaning.record else {
            unreachable!("admitted open")
        };
        open
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.meaning
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
impl std::ops::Deref for AdmittedWorldOpenAttempt {
    type Target = WorldOpenAttemptRecord;
    fn deref(&self) -> &Self::Target {
        self.open()
    }
}
