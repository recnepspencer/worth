use worth_store_physical_backend::{FilesystemBackendProfile, QualifiedMediaCapabilities};

fn invoke_from_profile(profile: &FilesystemBackendProfile) {
    profile.direct_io();
    profile.sparse_allocate(0, 4096);
}

fn promote_profile(profile: FilesystemBackendProfile) -> QualifiedMediaCapabilities {
    profile.into()
}

fn main() {}
