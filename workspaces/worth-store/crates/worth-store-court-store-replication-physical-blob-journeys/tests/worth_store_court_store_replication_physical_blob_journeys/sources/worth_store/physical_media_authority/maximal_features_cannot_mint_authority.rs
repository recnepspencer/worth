use worth_store::physical_runtime::MediaOwnedPhysicalRuntime;
use worth_store_physical_backend::physical_runtime::PhysicalStore;
use worth_store_physical_backend::{
    qualify_filesystem_media, FilesystemAccessPosture, FilesystemQualificationRequest,
};

fn backend_values_cannot_be_promoted(root: std::path::PathBuf) -> MediaOwnedPhysicalRuntime {
    let request = FilesystemQualificationRequest::production(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let backend_value = qualify_filesystem_media(request);
    backend_value.into()
}

fn main() {}
