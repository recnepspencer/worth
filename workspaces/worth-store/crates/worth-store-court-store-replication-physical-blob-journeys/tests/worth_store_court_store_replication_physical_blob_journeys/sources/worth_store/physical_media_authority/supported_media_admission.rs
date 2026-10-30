use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
};
use worth_store_physical_backend::FilesystemAccessPosture;

fn supported(root: std::path::PathBuf) {
    let admission = PhysicalRuntimeAdmission::new(root).unwrap();
    let runtime = PhysicalStore::admit(admission).unwrap();
    let media = FilesystemMediaAdmission::production(
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let _outcome = runtime.try_admit_filesystem_media(media);
}

fn main() {}
