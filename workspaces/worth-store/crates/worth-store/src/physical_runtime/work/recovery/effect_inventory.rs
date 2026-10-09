use worth_store_physical_backend::{ArtifactTreeDirectory, QualifiedFilesystemMedia};
use worth_store_physical_format::{
    physical_work_obligation::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
    store_namespace::StableStoreIdentity,
};

use super::{
    effect_obligation::journal_directory,
    integrity_admission::{admit_bounded_obligation, scope_from_pending_name},
    observation::{
        PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryAdmissionObservation,
        PhysicalWorkRecoveryIngressRejection,
    },
    PhysicalWorkRecoveryLocator,
};

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
    /// Reads the recovery-obligation journal a previous runtime left behind.
    pub(in crate::physical_runtime) fn inspect(
        media: &QualifiedFilesystemMedia,
        limit: usize,
    ) -> Self {
        let tree = media.artifact_tree();
        let directory = journal_directory();
        match tree.directory_exists(&directory) {
            Ok(false) => Self::empty(),
            Ok(true) => inspect_entries(media.store_identity(), tree, directory, limit),
            Err(failure) => Self::damaged(failure.kind()),
        }
    }

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
        if self.observations.len() != self.obligations.len() {
            return true;
        }
        self.obligations.iter().any(|obligation| {
            !matches!(
                obligation.target(),
                super::PhysicalWorkRecoveryTarget::ArtifactRemoval(
                    worth_store_physical_format::RecordArtifactFile::Segment { .. }
                )
            )
        })
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
