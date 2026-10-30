#![cfg_attr(test, allow(clippy::disallowed_methods))]
use worth_store_formal_models::assumptions::ModeledBackendDurabilityAssumption;
use worth_store_physical_backend::{
    AdversarialLostFlushProfile, BackendDurabilityProfile, PosixFileFsyncDirFsyncProfile,
};

#[test]
fn modeled_backend_assumptions_are_derived_from_runtime_profiles() {
    let posix =
        ModeledBackendDurabilityAssumption::from_runtime_profile::<PosixFileFsyncDirFsyncProfile>();
    assert_eq!(posix.runtime_profile(), PosixFileFsyncDirFsyncProfile::ID);
    assert_eq!(
        posix.required_barriers(),
        PosixFileFsyncDirFsyncProfile::REQUIRED_BARRIERS
    );
    assert_eq!(posix.support(), PosixFileFsyncDirFsyncProfile::SUPPORT);

    let hostile =
        ModeledBackendDurabilityAssumption::from_runtime_profile::<AdversarialLostFlushProfile>();
    assert_eq!(hostile.runtime_profile(), AdversarialLostFlushProfile::ID);
    assert_eq!(hostile.support(), AdversarialLostFlushProfile::SUPPORT);
}
