pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) const DROP_BOUNDARIES: [CertificationPhysicalMutationCheckpoint; 6] = [
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeEffectCutover,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringDataSettlement,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeTerminalFinalization,
];