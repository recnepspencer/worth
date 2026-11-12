//! The original submitted subject and its actual acknowledgement travel with
//! existing attempt resources through abandonment and retained materialization.
use super::{ActualWorldPublication, WorldDurableArtifact, WorldDurableMediumBinding};
use std::sync::{Arc, Mutex};
use worth_proof::DurablyCommitted;

pub(crate) struct WorldDurableOpenCustody {
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) subject: WorldDurableArtifact,
    admission: Mutex<OriginalOpenAdmission>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
enum OriginalOpenAdmission {
    Submitted(Option<DurablyCommitted<ActualWorldPublication>>),
    Readmitted {
        _source: super::AdmittedWorldOpenAttempt,
    },
}
impl std::fmt::Debug for WorldDurableOpenCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldDurableOpenCustody")
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}
impl WorldDurableOpenCustody {
    pub(crate) fn submitted(
        binding: Arc<WorldDurableMediumBinding>,
        subject: WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::WorldDurableMediumDenial> {
        Self::prepare(
            binding,
            subject,
            OriginalOpenAdmission::Submitted(None),
            execution,
        )
    }
    pub(crate) fn readmitted(
        binding: Arc<WorldDurableMediumBinding>,
        source: super::AdmittedWorldOpenAttempt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::WorldDurableMediumDenial> {
        let subject = WorldDurableArtifact::from_readmitted_open(&source, execution)?;
        Self::prepare(
            binding,
            subject,
            OriginalOpenAdmission::Readmitted { _source: source },
            execution,
        )
    }
    fn prepare(
        binding: Arc<WorldDurableMediumBinding>,
        subject: WorldDurableArtifact,
        admission: OriginalOpenAdmission,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, super::WorldDurableMediumDenial> {
        use super::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Frame::SizeLimit)?
            .0
            .pad_to_align();
        let bytes = u64::try_from(layout.size()).map_err(|_| Frame::SizeLimit)?;
        let funding = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        Ok(Arc::new(Self {
            binding,
            subject,
            admission: Mutex::new(admission),
            _funding: funding,
        }))
    }
    pub(crate) fn acknowledge(&self, committed: DurablyCommitted<ActualWorldPublication>) {
        // The binding submitted this subject and retains it until this exact
        // actual committed result is returned. Preserve that move-only proof.
        let mut admission = self
            .admission
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match &mut *admission {
            OriginalOpenAdmission::Submitted(original) => {
                assert!(original.is_none(), "one actual acknowledgement per open");
                *original = Some(committed);
            }
            OriginalOpenAdmission::Readmitted { .. } => {
                unreachable!("readmitted opens are never resubmitted")
            }
        }
    }
}
