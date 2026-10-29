use {worth_store_formal_models::assumptions::ModeledBackendDurabilityAssumption};
use {worth_store_physical_backend::AdversarialLostFlushProfile, worth_store_physical_backend::BackendDurabilityProfile, worth_store_physical_backend::PosixFileFsyncDirFsyncProfile};

#[test]
fn modeled_backend_assumptions_are_derived_from_runtime_profiles() {
    let posix =
        worth_store_formal_models::assumptions::ModeledBackendDurabilityAssumption::from_runtime_profile::<PosixFileFsyncDirFsyncProfile>();
    assert_eq!(posix.runtime_profile(), worth_store_physical_backend::PosixFileFsyncDirFsyncProfile::ID);
    assert_eq!(
        posix.required_barriers(),
        worth_store_physical_backend::PosixFileFsyncDirFsyncProfile::REQUIRED_BARRIERS
    );
    assert_eq!(posix.support(), worth_store_physical_backend::PosixFileFsyncDirFsyncProfile::SUPPORT);

    let hostile =
        worth_store_formal_models::assumptions::ModeledBackendDurabilityAssumption::from_runtime_profile::<AdversarialLostFlushProfile>();
    assert_eq!(hostile.runtime_profile(), worth_store_physical_backend::AdversarialLostFlushProfile::ID);
    assert_eq!(hostile.support(), worth_store_physical_backend::AdversarialLostFlushProfile::SUPPORT);
}
