//! The Home's single actual checkpoint attempt; returned observers own no work.
use std::sync::{Arc, Mutex, Weak};
use worth_relational::facade::durability::OpaqueCapturedCheckpointSubject;
use worth_runtime_world::facade::{
    PreparedWorldCheckpoint, WorldCheckpointSubject, WorldDurableMediumDenial,
};
use worth_store::physical_runtime::AnchoredRecordMembership;
use worth_store_relational::{PreparedRelationalSection, RelationalCheckpointSelectionSource};

#[derive(Default)]
pub(crate) struct HomeCheckpointOwner {
    pending: Mutex<Option<Arc<HomeCheckpointAttempt>>>,
    acknowledged_native: Mutex<[u64; 2]>,
}

pub(crate) struct HomeCheckpointAttempt {
    pub(super) world: Mutex<Option<Arc<PreparedWorldCheckpoint>>>,
    world_subject: WorldCheckpointSubject,
    manifest: Mutex<Option<HomeManifestPublication>>,
    pub(super) native: Mutex<Option<Arc<PreparedRelationalSection>>>,
    subject: Option<OpaqueCapturedCheckpointSubject>,
    receipt: Mutex<Option<AnchoredRecordMembership>>,
    pub(super) completed: Mutex<Option<super::ApplicationHomeCheckpoint>>,
    pub(super) input_memory: Mutex<Option<Arc<super::input_memory::CheckpointInputMemory>>>,
    pub(super) physical: Mutex<super::physical_publication::HomePhysicalPublication>,
    pub(super) wal: Mutex<super::physical_checkpoint::HomeWalCheckpoint>,
    _metadata_funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct HomeManifestPublication {
    pub(crate) generation: u64,
    pub(crate) payload_sha256: [u8; 32],
    pub(crate) payload_bytes: usize,
}

/// A description of work still owned by the Home. Dropping this value neither
/// abandons the actual attempt nor stops ordinary Native mutation.
#[derive(Clone)]
pub struct ApplicationHomeCheckpointRecovery {
    pub selected_generation: Option<u64>,
    pub acknowledgement_complete: bool,
    terminal: Option<Arc<worth_store::physical_runtime::AnchoredRecordStepTerminal>>,
    accepted_mutation: Option<worth_store::physical_runtime::PhysicalMutationIdentity>,
}

impl ApplicationHomeCheckpointRecovery {
    pub fn accepted_mutation(
        &self,
    ) -> Option<worth_store::physical_runtime::PhysicalMutationIdentity> {
        self.accepted_mutation
    }
    pub fn physical_terminal(
        &self,
    ) -> Option<&worth_store::physical_runtime::AnchoredRecordStepTerminal> {
        self.terminal.as_deref()
    }
}
impl std::fmt::Debug for ApplicationHomeCheckpointRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationHomeCheckpointRecovery")
            .field("selected_generation", &self.selected_generation)
            .field("acknowledgement_complete", &self.acknowledgement_complete)
            .field("physical_terminal", &self.terminal.is_some())
            .finish()
    }
}

impl HomeCheckpointOwner {
    /// Only the actual successful section readmission supplies this seed.
    /// Zero denotes no selected scheduling prefix, never absent owner state.
    pub(crate) fn admit_selected_native_coverage(&self, selected: Option<[u64; 2]>) {
        *self
            .acknowledged_native
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = selected.unwrap_or([0; 2]);
    }
    pub(crate) fn acknowledged_native(&self) -> [u64; 2] {
        *self
            .acknowledged_native
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
    pub(super) fn acknowledge_native(&self, coverage: [u64; 2]) {
        *self
            .acknowledged_native
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = coverage;
    }
    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn input_memory_bytes_for_test(&self) -> Option<u64> {
        self.pending()?
            .input_memory
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()?
            .bytes()
    }
    pub(super) fn pending(&self) -> Option<Arc<HomeCheckpointAttempt>> {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
    pub(super) fn install(
        &self,
        native: PreparedRelationalSection,
        world: PreparedWorldCheckpoint,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Arc<HomeCheckpointAttempt>,
            super::physical_publication::PublicationDriverLoan,
        ),
        super::ApplicationHomeCheckpointDenial,
    > {
        let quote = std::mem::size_of::<HomeCheckpointAttempt>()
            .checked_add(std::mem::size_of::<PreparedRelationalSection>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<PreparedWorldCheckpoint>()))
            .and_then(|bytes| bytes.checked_add(6 * std::mem::size_of::<usize>()))
            .ok_or(super::ApplicationHomeCheckpointDenial::SizeLimit)?;
        let funding = request
            .reserve_retained_memory(quote as u64)
            .map_err(super::ApplicationHomeCheckpointDenial::Execution)?;
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(attempt) = pending.as_ref() {
            return Err(super::ApplicationHomeCheckpointDenial::RecoveryRequired(
                attempt.observation(),
            ));
        }
        let attempt = Arc::new(HomeCheckpointAttempt {
            world_subject: world.publication_subject().clone(),
            world: Mutex::new(Some(Arc::new(world))),
            manifest: Mutex::new(None),
            subject: native.publication_subject().cloned(),
            native: Mutex::new(Some(Arc::new(native))),
            receipt: Mutex::new(None),
            completed: Mutex::new(None),
            input_memory: Mutex::new(None),
            physical: Mutex::new(Default::default()),
            wal: Mutex::new(Default::default()),
            _metadata_funding: funding,
        });
        let driver = attempt
            .try_driver()
            .expect("unexposed attempt has its initial driver");
        *pending = Some(attempt.clone());
        Ok((attempt, driver))
    }
    pub(super) fn clear(&self, attempt: &Arc<HomeCheckpointAttempt>) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if pending
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, attempt))
        {
            pending.take();
        }
    }
    pub(crate) fn selection_source(
        self: &Arc<Self>,
    ) -> Arc<dyn RelationalCheckpointSelectionSource> {
        Arc::new(HomeCheckpointSelectionSource {
            owner: Arc::downgrade(self),
        })
    }
    pub(crate) fn with_selected_world<R>(
        &self,
        subject: &WorldCheckpointSubject,
        consume: impl FnOnce(
            &PreparedWorldCheckpoint,
            HomeManifestPublication,
            &AnchoredRecordMembership,
        ) -> R,
    ) -> Result<Option<R>, WorldDurableMediumDenial> {
        let Some(attempt) = self.pending() else {
            return Ok(None);
        };
        if !attempt.world_subject.same_capture(subject) {
            return Ok(None);
        }
        let world = attempt
            .world
            .try_lock()
            .map_err(|_| WorldDurableMediumDenial::PublicationBusy)?;
        let manifest = attempt
            .manifest
            .try_lock()
            .map_err(|_| WorldDurableMediumDenial::PublicationBusy)?;
        let receipt = attempt
            .receipt
            .try_lock()
            .map_err(|_| WorldDurableMediumDenial::PublicationBusy)?;
        let (Some(world), Some(manifest), Some(receipt)) =
            (world.as_ref(), *manifest, receipt.as_ref())
        else {
            return Ok(None);
        };
        Ok(Some(consume(world, manifest, receipt)))
    }
}

impl HomeCheckpointAttempt {
    pub(super) fn bind_manifest(
        &self,
        manifest: HomeManifestPublication,
    ) -> Result<(), super::ApplicationHomeCheckpointDenial> {
        let mut original = self
            .manifest
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if original.is_some_and(|before| before != manifest) {
            return Err(super::ApplicationHomeCheckpointDenial::World(
                WorldDurableMediumDenial::StaleCheckpoint,
            ));
        }
        *original = Some(manifest);
        Ok(())
    }
    pub(super) fn world(&self) -> Option<Arc<PreparedWorldCheckpoint>> {
        self.world
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
    pub(super) fn selected(&self, receipt: AnchoredRecordMembership) {
        *self
            .receipt
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(receipt);
    }
    pub(super) fn native(&self) -> Option<Arc<PreparedRelationalSection>> {
        self.native
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
    pub(super) fn finish(&self, result: super::ApplicationHomeCheckpoint) {
        *self
            .completed
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(result);
        self.clear_settled_payload();
    }
    pub(super) fn clear_settled_payload(&self) {
        let world = self
            .world
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        let native = self
            .native
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        let input = self
            .input_memory
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        self.physical
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        drop(world);
        drop(native);
        drop(input);
    }
    pub(super) fn observation(&self) -> ApplicationHomeCheckpointRecovery {
        let (accepted_mutation, terminal) = {
            let physical = self
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            (
                physical
                    .registration
                    .as_ref()
                    .map(|registration| registration.mutation_identity()),
                physical.terminal(),
            )
        };
        ApplicationHomeCheckpointRecovery {
            selected_generation: self
                .receipt
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref()
                .map(|receipt| receipt.version().generation()),
            acknowledgement_complete: self
                .completed
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .is_some(),
            terminal,
            accepted_mutation,
        }
    }
}

impl Drop for HomeCheckpointOwner {
    fn drop(&mut self) {
        if let Some(attempt) = self
            .pending
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            attempt.stop_advancement();
        }
    }
}

struct HomeCheckpointSelectionSource {
    owner: Weak<HomeCheckpointOwner>,
}
impl std::fmt::Debug for HomeCheckpointSelectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HomeCheckpointSelectionSource")
    }
}
impl RelationalCheckpointSelectionSource for HomeCheckpointSelectionSource {
    fn selected_membership(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership> {
        let owner = self.owner.upgrade()?;
        let attempt = owner.pending()?;
        if !attempt.subject.as_ref()?.same_capture(subject) {
            return None;
        }
        let receipt = attempt
            .receipt
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        receipt
    }
}
