use std::sync::Mutex;

use worth_store_physical_backend::{
    ArtifactTreeDirectory, ArtifactTreeFile, QualifiedFilesystemMedia,
};
use worth_store_physical_format::{
    physical_work_obligation::{
        encode_physical_work_obligation_v6, PhysicalWorkObligationV6,
        PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
    },
    store_namespace::StableStoreIdentity,
};

use super::{
    super::{PhysicalWorkIdentity, PhysicalWorkOperationFamily},
    format_mapping::{operation_to_format, target_to_format},
    integrity_admission::{admit_bounded_obligation, scope_from_pending_name},
    observation::{
        PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryAdmissionObservation,
        PhysicalWorkRecoveryIngressRejection,
    },
    PhysicalWorkRecoveryLocator, PhysicalWorkRecoveryTarget,
};

pub(in crate::physical_runtime) struct PhysicalEffectJournal {
    directory: ArtifactTreeDirectory,
    initialized: Mutex<bool>,
}

pub(in crate::physical_runtime) struct PreparedPhysicalEffect {
    artifact: ArtifactTreeFile,
}

pub(in crate::physical_runtime) struct PhysicalEffectRecoveryInventory {
    obligations: Box<[PhysicalWorkRecoveryLocator]>,
    observations: Box<[PhysicalWorkRecoveryAdmissionObservation]>,
    counters: PhysicalWorkRecoveryAdmissionCounters,
}

enum PhysicalWorkEntryInspection {
    Admitted {
        locator: PhysicalWorkRecoveryLocator,
        observation: PhysicalWorkRecoveryAdmissionObservation,
    },
    Rejected(PhysicalWorkRecoveryAdmissionObservation),
}

impl PhysicalEffectJournal {
    pub(in crate::physical_runtime) fn new(media: &QualifiedFilesystemMedia) -> Self {
        let directory = journal_directory();
        let initialized = media
            .artifact_tree()
            .directory_exists(&directory)
            .unwrap_or(false);
        Self {
            directory,
            initialized: Mutex::new(initialized),
        }
    }

    pub(in crate::physical_runtime) fn inspect(
        media: &QualifiedFilesystemMedia,
        limit: usize,
    ) -> PhysicalEffectRecoveryInventory {
        let tree = media.artifact_tree();
        let directory = journal_directory();
        match tree.directory_exists(&directory) {
            Ok(false) => PhysicalEffectRecoveryInventory::empty(),
            Ok(true) => inspect_entries(media.store_identity(), tree, directory, limit),
            Err(failure) => PhysicalEffectRecoveryInventory::damaged(failure.kind()),
        }
    }

    pub(in crate::physical_runtime) fn prepare(
        &self,
        media: &QualifiedFilesystemMedia,
        identity: PhysicalWorkIdentity,
        operation: PhysicalWorkOperationFamily,
        target: PhysicalWorkRecoveryTarget,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<PreparedPhysicalEffect, ()> {
        self.ensure_directory(media)?;
        let artifact = self
            .directory
            .file(&format!(
                "effect-{:016x}-{:016x}-{:016x}.pending",
                identity.runtime().get(),
                identity.generation().lifecycle().get(),
                identity.operation().get(),
            ))
            .map_err(|_| ())?;
        let record = encode_record(identity, operation, target, payload_digest);
        let tree = media.artifact_tree();
        tree.write_new_obligation_record(&artifact, &record)
            .map_err(|_| ())?;
        tree.synchronize_directory(&self.directory)
            .map_err(|_| ())?;
        Ok(PreparedPhysicalEffect { artifact })
    }

    pub(in crate::physical_runtime) fn finish(
        &self,
        media: &QualifiedFilesystemMedia,
        prepared: PreparedPhysicalEffect,
    ) -> Result<(), ()> {
        media
            .artifact_tree()
            .remove_file_durably(&prepared.artifact)
            .map_err(|_| ())
    }

    fn ensure_directory(&self, media: &QualifiedFilesystemMedia) -> Result<(), ()> {
        let mut initialized = self
            .initialized
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *initialized {
            return Ok(());
        }
        let tree = media.artifact_tree();
        if !tree.directory_exists(&self.directory).map_err(|_| ())? {
            tree.create_directory(&self.directory).map_err(|_| ())?;
            tree.synchronize_directory(&ArtifactTreeDirectory::families())
                .map_err(|_| ())?;
        }
        *initialized = true;
        Ok(())
    }
}

fn journal_directory() -> ArtifactTreeDirectory {
    ArtifactTreeDirectory::families()
        .child("physical-work")
        .expect("portable physical-work recovery path")
}

pub(super) fn encode_record(
    identity: PhysicalWorkIdentity,
    operation: PhysicalWorkOperationFamily,
    target: PhysicalWorkRecoveryTarget,
    payload_digest: Option<[u8; 32]>,
) -> [u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES] {
    let value = PhysicalWorkObligationV6::new(
        identity.store().bytes(),
        identity.runtime().get(),
        identity.generation().lifecycle().get(),
        identity.operation().get(),
        operation_to_format(operation),
        target_to_format(target),
        payload_digest,
    )
    .expect("Store physical-work identity and target satisfy v6 format");
    encode_physical_work_obligation_v6(value)
}

fn inspect_entries(
    store: StableStoreIdentity,
    tree: worth_store_physical_backend::ArtifactTreeMedia<'_>,
    directory: ArtifactTreeDirectory,
    limit: usize,
) -> PhysicalEffectRecoveryInventory {
    let names = match tree.list_file_names_bounded(&directory, limit) {
        Ok(names) => names,
        Err(failure) => return PhysicalEffectRecoveryInventory::damaged(failure.kind()),
    };
    let mut obligations = Vec::with_capacity(names.len());
    let mut observations = Vec::with_capacity(names.len());
    let mut counters = PhysicalWorkRecoveryAdmissionCounters::default();
    for name in names {
        match inspect_entry(store, &tree, &directory, &name, &mut counters) {
            PhysicalWorkEntryInspection::Admitted {
                locator,
                observation,
            } => {
                obligations.push(locator);
                observations.push(observation);
            }
            PhysicalWorkEntryInspection::Rejected(observation) => observations.push(observation),
        }
    }
    PhysicalEffectRecoveryInventory {
        obligations: obligations.into_boxed_slice(),
        observations: observations.into_boxed_slice(),
        counters,
    }
}

fn inspect_entry(
    store: StableStoreIdentity,
    tree: &worth_store_physical_backend::ArtifactTreeMedia<'_>,
    directory: &ArtifactTreeDirectory,
    name: &str,
    counters: &mut PhysicalWorkRecoveryAdmissionCounters,
) -> PhysicalWorkEntryInspection {
    counters.attempt();
    let scope = match scope_from_pending_name(store, name) {
        Ok(scope) => scope,
        Err(rejection) => {
            counters.rejected_before_owner_interpretation();
            return rejected_entry(name, None, rejection);
        }
    };
    let file = match directory.file(name) {
        Ok(file) => file,
        Err(_) => {
            counters.rejected_before_owner_interpretation();
            return rejected_entry(
                name,
                Some(scope),
                PhysicalWorkRecoveryIngressRejection::InvalidPendingName,
            );
        }
    };
    let record = match tree.read_bounded(&file, PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64) {
        Ok(record) => record,
        Err(failure) => {
            counters.rejected_before_owner_interpretation();
            return rejected_entry(
                name,
                Some(scope),
                PhysicalWorkRecoveryIngressRejection::ReadFailure(failure.kind()),
            );
        }
    };
    match admit_bounded_obligation(scope, &record, counters) {
        Ok(locator) => PhysicalWorkEntryInspection::Admitted {
            locator,
            observation: PhysicalWorkRecoveryAdmissionObservation::admitted(name, scope),
        },
        Err(rejection) => rejected_entry(name, Some(scope), rejection),
    }
}

fn rejected_entry(
    name: &str,
    scope: Option<worth_store_physical_integrity::PhysicalArtifactScope>,
    rejection: PhysicalWorkRecoveryIngressRejection,
) -> PhysicalWorkEntryInspection {
    PhysicalWorkEntryInspection::Rejected(PhysicalWorkRecoveryAdmissionObservation::rejected(
        name, scope, rejection,
    ))
}

impl PhysicalEffectRecoveryInventory {
    fn empty() -> Self {
        Self {
            obligations: Box::new([]),
            observations: Box::new([]),
            counters: PhysicalWorkRecoveryAdmissionCounters::default(),
        }
    }

    fn damaged(failure: worth_store_physical_backend::ArtifactTreeFailureKind) -> Self {
        Self {
            obligations: Box::new([]),
            observations: Box::from([
                PhysicalWorkRecoveryAdmissionObservation::inventory_rejected(
                    PhysicalWorkRecoveryIngressRejection::ReadFailure(failure),
                ),
            ]),
            counters: PhysicalWorkRecoveryAdmissionCounters::default(),
        }
    }

    pub(in crate::physical_runtime) fn requires_inspection(&self) -> bool {
        !self.observations.is_empty()
    }

    pub(in crate::physical_runtime) fn obligations(&self) -> &[PhysicalWorkRecoveryLocator] {
        &self.obligations
    }

    pub(in crate::physical_runtime) const fn evidence_damaged(&self) -> bool {
        self.observations.len() != self.obligations.len()
    }

    pub(in crate::physical_runtime) fn admission_observations(
        &self,
    ) -> &[PhysicalWorkRecoveryAdmissionObservation] {
        &self.observations
    }

    pub(in crate::physical_runtime) const fn admission_counters(
        &self,
    ) -> PhysicalWorkRecoveryAdmissionCounters {
        self.counters
    }
}
