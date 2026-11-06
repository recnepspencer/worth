use super::{AdmittedWorldOpenAttempt, RecoveredCustodyDisposition};
use crate::durability::{WorldCommittedPublication, WorldDurableArtifact, WorldRecord};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Original metadata and backing are admitted before durable release submits.
#[derive(Debug)]
pub(crate) struct PreparedWorldReleasedDisposition {
    open: AdmittedWorldOpenAttempt,
    value: RecoveredCustodyDisposition,
    subject: WorldDurableArtifact,
    temporary: Option<ExecutionMemoryReservation>,
}
impl PreparedWorldReleasedDisposition {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        open: &AdmittedWorldOpenAttempt,
        subject: &WorldDurableArtifact,
    ) -> Result<Box<Self>, crate::durability::WorldDurableMediumDenial> {
        use crate::durability::WorldDurableMediumDenial as Denial;
        let WorldRecord::Released {
            association,
            custody,
            relational,
        } = subject.record()
        else {
            return Err(Denial::PublicationUnresolved);
        };
        if association != &open.association || *custody != open.custody {
            return Err(Denial::PublicationUnresolved);
        }
        let temporary_bytes = std::mem::size_of::<Self>()
            .checked_add(open.expected.branch.name().len())
            .ok_or(Denial::Unavailable)?;
        let temporary = request
            .reserve_retained_memory(temporary_bytes as u64)
            .map_err(Denial::Request)?;
        Ok(Box::new(Self {
            open: open.clone(),
            value: RecoveredCustodyDisposition::Released(relational.clone()),
            subject: subject.clone(),
            temporary,
        }))
    }
    pub(crate) fn seal(
        self: Box<Self>,
        committed: &WorldCommittedPublication,
    ) -> AdmittedWorldDisposition {
        assert!(
            self.subject.same_subject(committed.subject()),
            "same original release was acknowledged"
        );
        let Self {
            open,
            value,
            subject,
            temporary,
        } = *self;
        drop(subject);
        drop(temporary);
        AdmittedWorldDisposition::new(
            open,
            value,
            committed.position(),
            *committed.digest().bytes(),
        )
    }
}

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
