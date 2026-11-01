use super::{AdmittedWorldOpenAttempt, RecoveredCustodyDisposition};

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldDisposition {
    open: AdmittedWorldOpenAttempt,
    value: RecoveredCustodyDisposition,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldDisposition {
    pub(super) fn new(
        open: AdmittedWorldOpenAttempt,
        value: RecoveredCustodyDisposition,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            open,
            value,
            position,
            digest,
        }
    }
    pub(crate) fn disposition(&self) -> &RecoveredCustodyDisposition {
        &self.value
    }
    pub(crate) fn open(&self) -> &AdmittedWorldOpenAttempt {
        &self.open
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
