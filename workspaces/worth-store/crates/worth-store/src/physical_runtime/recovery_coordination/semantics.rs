use worth_foundational::{
    aspects, AspectContract, AspectMask, AspectValue, ContractValidationInput, InternedString,
    MutationMask, ProjectionMask, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_signal::facade::PartitionSubscription;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectContractAdmission,
    StoreAspectIdentity, StoreAspectPatchAuthorityInput, StoreAspectPatchBoundaryFact,
    StorePhysicalBoundaryWitness,
};

use crate::physical_runtime::{
    PhysicalSignalAspectDeclaration, PhysicalSignalAspectRole, PhysicalWorkCapacity,
    PhysicalWorkProfileDeclaration, PhysicalWorkSemanticBasis, PhysicalWorkSignalFamily,
    PhysicalWorkSignalFamilySet,
};

const REVISION: u64 = 1;
const CONTRACTS: [(
    &str,
    u64,
    PhysicalSignalAspectRole,
    PhysicalWorkSignalFamily,
); 5] = [
    (
        "store.physical.recovery.discovery-basis",
        1_401,
        PhysicalSignalAspectRole::Dependency,
        PhysicalWorkSignalFamily::ReadFault,
    ),
    (
        "store.physical.recovery.redo-basis",
        1_402,
        PhysicalSignalAspectRole::DependencyAndOutput,
        PhysicalWorkSignalFamily::ExactWriteback,
    ),
    (
        "store.physical.recovery.publication-basis",
        1_403,
        PhysicalSignalAspectRole::DependencyAndOutput,
        PhysicalWorkSignalFamily::RootPublication,
    ),
    (
        "store.physical.recovery.cleanup-basis",
        1_404,
        PhysicalSignalAspectRole::DependencyAndOutput,
        PhysicalWorkSignalFamily::WalReclamation,
    ),
    (
        "store.physical.recovery.checkpoint-residue-basis",
        1_405,
        PhysicalSignalAspectRole::DependencyAndOutput,
        PhysicalWorkSignalFamily::CheckpointCapture,
    ),
];

pub(super) struct InstalledRecoverySemantics {
    pub(super) profile: PhysicalWorkProfileDeclaration,
    pub(super) work_security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    pub(super) scheduler_security: worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    pub(super) bases: [PhysicalWorkSemanticBasis; 5],
}

pub(super) fn install(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    session: [u8; 16],
    capacity: PhysicalWorkCapacity,
) -> InstalledRecoverySemantics {
    let witness = crate::physical_runtime::record_serving::work_semantics::security_admission::
        physical_witness();
    let partitions = recovery_partitions(store, session);
    let installed = std::array::from_fn(|index| {
        install_contract(CONTRACTS[index], partitions[index].clone(), witness)
    });
    let [discovery, redo, publication, cleanup, checkpoint_residue] = installed;
    let authority_fact = discovery
        .basis
        .projection_fact()
        .expect("discovery semantics are projection authority");
    let (security, scheduler_security) =
        crate::physical_runtime::record_serving::work_semantics::security_admission::
            admit_scheduler_scope(authority_fact);
    let profile = PhysicalWorkProfileDeclaration::from_signal_aspects(
        security,
        [
            discovery.declaration,
            redo.declaration,
            publication.declaration,
            cleanup.declaration,
            checkpoint_residue.declaration,
        ],
    )
    .expect("the distinct recovery contracts form one bounded native profile")
    .with_capacity(capacity);
    InstalledRecoverySemantics {
        profile,
        work_security: security,
        scheduler_security,
        bases: [
            discovery.basis,
            redo.basis,
            publication.basis,
            cleanup.basis,
            checkpoint_residue.basis,
        ],
    }
}

struct InstalledContract {
    basis: PhysicalWorkSemanticBasis,
    declaration: PhysicalSignalAspectDeclaration,
}

fn install_contract(
    (key, identity, role, family): (
        &'static str,
        u64,
        PhysicalSignalAspectRole,
        PhysicalWorkSignalFamily,
    ),
    partition: String,
    witness: StorePhysicalBoundaryWitness,
) -> InstalledContract {
    let (contract, aspect, admission) = contract(key, identity, role, witness);
    let basis = match role {
        PhysicalSignalAspectRole::Dependency => projection_basis(
            &contract,
            aspect,
            admission.clone(),
            witness,
            partition.clone(),
        ),
        PhysicalSignalAspectRole::Output | PhysicalSignalAspectRole::DependencyAndOutput => {
            let (basis, _) = mutation_basis(
                &contract,
                aspect,
                admission.clone(),
                witness,
                partition.clone(),
            );
            basis
        }
    };
    let families = PhysicalWorkSignalFamilySet::only(family);
    let families = if key == "store.physical.recovery.redo-basis" {
        families.with(PhysicalWorkSignalFamily::Publication)
    } else {
        families
    };
    let declaration = PhysicalSignalAspectDeclaration::new(admission, role)
        .for_families(families)
        .with_partition(PartitionSubscription::whole_partition(partition));
    InstalledContract { basis, declaration }
}

fn contract(
    key: &'static str,
    identity: u64,
    role: PhysicalSignalAspectRole,
    witness: StorePhysicalBoundaryWitness,
) -> (
    AspectContract,
    StoreAspectIdentity,
    StoreAspectContractAdmission,
) {
    let key = aspects()
        .vocabulary()
        .key(key)
        .expect("recovery key is canonical");
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(identity))
        .at_revision(aspects().vocabulary().revision(REVISION))
        .scalar(ScalarAspectType::String);
    let aspect = StoreAspectIdentity::from_aspect_key(key);
    let admission = StoreAspectContractAdmission::new(aspect.clone(), contract.clone(), witness)
        .expect("recovery contract identity matches its key")
        .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
        .expect("recovery contract admits whole projection");
    let admission = if role == PhysicalSignalAspectRole::Dependency {
        admission
    } else {
        admission
            .admit_mutation_mask(AspectMask::<MutationMask>::whole_aspect())
            .expect("recovery mutation contract admits whole mutation")
    };
    (contract, aspect, admission)
}

fn projection_basis(
    contract: &AspectContract,
    identity: StoreAspectIdentity,
    admission: StoreAspectContractAdmission,
    witness: StorePhysicalBoundaryWitness,
    value: String,
) -> PhysicalWorkSemanticBasis {
    let state = match aspects()
        .authoritative_state()
        .admit([validated_value(contract, value)])
    {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("recovery discovery state must admit: {outcome:?}"),
    };
    let fact = StoreAspectBoundaryFact::from_admitted_state(
        identity,
        StoreAspectAuthorityInput::new(state, witness),
    )
    .expect("recovery discovery fact targets its exact contract");
    PhysicalWorkSemanticBasis::projection(fact, admission)
        .expect("recovery discovery fact and contract are constructed together")
}

fn mutation_basis(
    contract: &AspectContract,
    identity: StoreAspectIdentity,
    admission: StoreAspectContractAdmission,
    witness: StorePhysicalBoundaryWitness,
    value: String,
) -> (PhysicalWorkSemanticBasis, StoreAspectPatchBoundaryFact) {
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(contract, value))
        .finish()
    {
        TransitionOutcome::Success(patch) => patch,
        outcome => panic!("recovery mutation patch must admit: {outcome:?}"),
    };
    let fact = StoreAspectPatchBoundaryFact::from_authoritative_patch(
        identity,
        StoreAspectPatchAuthorityInput::new(patch, witness),
    )
    .expect("recovery mutation fact targets its exact contract");
    let basis = PhysicalWorkSemanticBasis::mutation(fact.clone(), admission)
        .expect("recovery mutation fact and contract are constructed together");
    (basis, fact)
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
        outcome => panic!("recovery aspect value must validate: {outcome:?}"),
    }
}

fn recovery_partitions(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    session: [u8; 16],
) -> [String; 5] {
    let store = store
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let session = session
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    [
        "discovery",
        "redo",
        "publication",
        "cleanup",
        "checkpoint-residue",
    ]
    .map(|stage| format!("store.physical.recovery/{store}/{session}/{stage}"))
}
