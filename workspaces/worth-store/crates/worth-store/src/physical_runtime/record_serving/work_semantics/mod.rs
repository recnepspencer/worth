use worth_foundational::{
    aspects, AspectContract, AspectMask, AspectValue, ContractValidationInput, InternedString,
    MutationMask, ProjectionMask, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectContractAdmission, StoreAspectIdentity, StoreAspectPatchAuthorityInput,
    StoreAspectPatchBoundaryFact, StorePhysicalBoundaryWitness,
};

use crate::physical_runtime::work::{
    PhysicalSignalAspectDeclaration, PhysicalSignalAspectRole, PhysicalWorkProfileDeclaration,
    PhysicalWorkProfileDenial, PhysicalWorkSemanticBasis, PhysicalWorkSignalFamily,
    PhysicalWorkSignalFamilySet,
};

use super::RecordPublicationStage;

mod durability;
mod frame_writeback_basis;
pub(in crate::physical_runtime) mod integrity_admission;
mod publication_basis;
mod read_basis;
pub(in crate::physical_runtime) mod security_admission;

const RECORD_ASPECT_REVISION: u64 = 1;

pub(in crate::physical_runtime) use read_basis::RecordReadPartition;
use read_basis::RecordReadSemanticBases;

#[derive(Clone, Debug)]
pub(in crate::physical_runtime) struct RecordWorkAdmission {
    security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    scheduler_security: worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    read_bases: RecordReadSemanticBases,
    publication_bases: publication_basis::RecordPublicationSemanticBases,
    frame_writeback_basis: PhysicalWorkSemanticBasis,
    durability_policy_basis: PhysicalWorkSemanticBasis,
    root_publication_basis: PhysicalWorkSemanticBasis,
    checkpoint_capture_basis: PhysicalWorkSemanticBasis,
    wal_append_basis: PhysicalWorkSemanticBasis,
    wal_barrier_basis: PhysicalWorkSemanticBasis,
    wal_reclamation_basis: PhysicalWorkSemanticBasis,
}

impl RecordWorkAdmission {
    pub(in crate::physical_runtime) fn install(
        profile: PhysicalWorkProfileDeclaration,
        durability: crate::physical_runtime::PhysicalDurabilityObservation,
    ) -> Result<(Self, PhysicalWorkProfileDeclaration), PhysicalWorkProfileDenial> {
        let witness = security_admission::physical_witness();
        let read = read_basis::install(witness);
        let publication = publication_basis::install(witness);
        let frame_writeback = frame_writeback_basis::install(witness);
        let durability_policy =
            durability::install_policy_binding(witness, durability.policy_identity());
        let store_partition = durability_store_partition(durability);
        let root_publication =
            durability::install_root_publication(witness, store_partition.clone());
        let checkpoint_capture =
            durability::install_checkpoint_capture(witness, store_partition.clone());
        let wal_append = durability::install_wal_append(witness, store_partition.clone());
        let wal_barrier = durability::install_wal_barrier(witness, store_partition.clone());
        let wal_reclamation = durability::install_wal_reclamation(witness, store_partition);
        let extensions = read.declarations.into_iter().chain([
            publication.declaration,
            frame_writeback.declaration,
            durability_policy.declaration,
            root_publication.declaration,
            checkpoint_capture.declaration,
            wal_append.declaration,
            wal_barrier.declaration,
            wal_reclamation.declaration,
        ]);
        let profile = profile.with_native_extensions(read.security, extensions)?;
        Ok((
            Self {
                security: read.security,
                scheduler_security: read.scheduler_security,
                read_bases: read.bases,
                publication_bases: publication.bases,
                frame_writeback_basis: frame_writeback.basis,
                durability_policy_basis: durability_policy.basis,
                root_publication_basis: root_publication.basis,
                checkpoint_capture_basis: checkpoint_capture.basis,
                wal_append_basis: wal_append.basis,
                wal_barrier_basis: wal_barrier.basis,
                wal_reclamation_basis: wal_reclamation.basis,
            },
            profile,
        ))
    }

    pub(in crate::physical_runtime) const fn security(
        &self,
    ) -> worth_store_security::StoreAuthorityBoundSecurityScopeReceipt {
        self.security
    }

    pub(in crate::physical_runtime) const fn scheduler_security(
        &self,
    ) -> &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission {
        &self.scheduler_security
    }

    pub(in crate::physical_runtime) fn read_basis(
        &self,
        partition: RecordReadPartition,
    ) -> PhysicalWorkSemanticBasis {
        self.read_bases.for_partition(partition)
    }

    pub(in crate::physical_runtime) fn mutation_basis(
        &self,
        stage: RecordPublicationStage,
    ) -> PhysicalWorkSemanticBasis {
        self.publication_bases.for_stage(stage)
    }

    pub(in crate::physical_runtime) fn frame_writeback_basis(&self) -> PhysicalWorkSemanticBasis {
        self.frame_writeback_basis.clone()
    }

    pub(in crate::physical_runtime) fn durability_policy_basis(&self) -> PhysicalWorkSemanticBasis {
        self.durability_policy_basis.clone()
    }

    pub(in crate::physical_runtime) fn root_publication_basis(&self) -> PhysicalWorkSemanticBasis {
        self.root_publication_basis.clone()
    }

    pub(in crate::physical_runtime) fn checkpoint_capture_basis(
        &self,
    ) -> PhysicalWorkSemanticBasis {
        self.checkpoint_capture_basis.clone()
    }

    pub(in crate::physical_runtime) fn wal_append_basis(&self) -> PhysicalWorkSemanticBasis {
        self.wal_append_basis.clone()
    }

    pub(in crate::physical_runtime) fn wal_barrier_basis(&self) -> PhysicalWorkSemanticBasis {
        self.wal_barrier_basis.clone()
    }

    pub(in crate::physical_runtime) fn wal_reclamation_basis(&self) -> PhysicalWorkSemanticBasis {
        self.wal_reclamation_basis.clone()
    }
}

fn dependency_and_output_declaration(
    admission: StoreAspectContractAdmission,
    family: PhysicalWorkSignalFamily,
) -> PhysicalSignalAspectDeclaration {
    PhysicalSignalAspectDeclaration::new(admission, PhysicalSignalAspectRole::DependencyAndOutput)
        .for_families(PhysicalWorkSignalFamilySet::only(family))
}

fn partitioned_dependency_and_output_declaration(
    admission: StoreAspectContractAdmission,
    family: PhysicalWorkSignalFamily,
    partition: String,
) -> PhysicalSignalAspectDeclaration {
    PhysicalSignalAspectDeclaration::new(admission, PhysicalSignalAspectRole::DependencyAndOutput)
        .for_families(PhysicalWorkSignalFamilySet::only(family))
        .with_partition(worth_signal::facade::PartitionSubscription::whole_partition(partition))
}

fn durability_store_partition(
    durability: crate::physical_runtime::PhysicalDurabilityObservation,
) -> String {
    let store = durability
        .store_identity()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("physical-durability-store/{store}")
}

fn contract(
    key: &'static str,
    contract_identity: u64,
    witness: StorePhysicalBoundaryWitness,
) -> (
    AspectContract,
    StoreAspectIdentity,
    StoreAspectContractAdmission,
) {
    let key = aspects()
        .vocabulary()
        .key(key)
        .expect("built-in record aspect key is canonical");
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(contract_identity))
        .at_revision(aspects().vocabulary().revision(RECORD_ASPECT_REVISION))
        .scalar(ScalarAspectType::String);
    let identity = StoreAspectIdentity::from_aspect_key(key);
    let admission = StoreAspectContractAdmission::new(identity.clone(), contract.clone(), witness)
        .expect("built-in record contract identity matches its key")
        .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
        .expect("built-in scalar record contract admits whole projection")
        .admit_mutation_mask(AspectMask::<MutationMask>::whole_aspect())
        .expect("built-in scalar record contract admits whole mutation");
    (contract, identity, admission)
}

fn projection_contract(
    key: &'static str,
    contract_identity: u64,
    witness: StorePhysicalBoundaryWitness,
) -> (
    AspectContract,
    StoreAspectIdentity,
    StoreAspectContractAdmission,
) {
    let key = aspects()
        .vocabulary()
        .key(key)
        .expect("built-in record aspect key is canonical");
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(contract_identity))
        .at_revision(aspects().vocabulary().revision(RECORD_ASPECT_REVISION))
        .scalar(ScalarAspectType::String);
    let identity = StoreAspectIdentity::from_aspect_key(key);
    let admission = StoreAspectContractAdmission::new(identity.clone(), contract.clone(), witness)
        .expect("built-in record contract identity matches its key")
        .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
        .expect("built-in scalar record contract admits whole projection");
    (contract, identity, admission)
}

fn patch_fact(
    contract: &AspectContract,
    identity: StoreAspectIdentity,
    witness: StorePhysicalBoundaryWitness,
    value: &'static str,
) -> StoreAspectPatchBoundaryFact {
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(contract, value))
        .finish()
    {
        TransitionOutcome::Success(patch) => patch,
        outcome => panic!("built-in record aspect patch must admit: {outcome:?}"),
    };
    StoreAspectPatchBoundaryFact::from_authoritative_patch(
        identity,
        StoreAspectPatchAuthorityInput::new(patch, witness),
    )
    .expect("built-in record patch targets exactly its declared identity")
}

fn validated_value(
    contract: &AspectContract,
    value: impl Into<InternedString>,
) -> worth_foundational::ContractValidatedAspectArtifact {
    match aspects()
        .validate()
        .against(contract)
        .value(ContractValidationInput::from(AspectValue::String(
            value.into(),
        ))) {
        TransitionOutcome::Success(value) => value,
        outcome => panic!("built-in record aspect value must validate: {outcome:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_and_frame_writeback_are_distinct_authorities() {
        let witness = security_admission::physical_witness();
        let publication = publication_basis::install(witness);
        let writeback = frame_writeback_basis::install(witness);
        assert_ne!(publication.bases.candidate_data, writeback.basis);
        assert_eq!(
            publication.declaration.families(),
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::Publication)
        );
        assert_eq!(
            writeback.declaration.families(),
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::ExactWriteback)
        );
    }
}
