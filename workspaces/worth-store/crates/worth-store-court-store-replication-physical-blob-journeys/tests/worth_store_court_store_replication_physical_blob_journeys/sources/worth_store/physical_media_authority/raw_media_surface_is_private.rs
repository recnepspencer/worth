use worth_store_physical_backend::{
    FilesystemMediaAdmissionAuthority, NamespaceFileHandle, NamespacePublicationTarget,
    NamespaceRelativePath, PositionedWriteRequest, QualifiedFilesystemMedia, ReadOnlyFileAccess,
    StagedNamespacePath,
};
use worth_store_physical_format::store_namespace::{
    NamespaceInitializationAttempt, StagedNamespaceName, StoreNamespaceRelativeRole,
};

fn forge_admission_authority() -> FilesystemMediaAdmissionAuthority {
    FilesystemMediaAdmissionAuthority::for_test()
}

fn extract_raw_owner(media: QualifiedFilesystemMedia) {
    let _ = media.into_runtime_parts();
}

fn raw_paths_cannot_mint_capabilities() {
    let _ = NamespaceRelativePath::for_role(StoreNamespaceRelativeRole::IdentityRecord);
    let attempt = NamespaceInitializationAttempt::from_nonzero_bytes([1; 16]).unwrap();
    let staged = StagedNamespaceName::for_identity(attempt);
    let _ = StagedNamespacePath::for_identity(&staged);
    let _ = NamespacePublicationTarget::identity_record();
}

fn read_handle_cannot_mutate(handle: &NamespaceFileHandle<'_, ReadOnlyFileAccess>) {
    let _ = handle.positioned_write(PositionedWriteRequest::new(0, b"forbidden"));
}

fn main() {}
