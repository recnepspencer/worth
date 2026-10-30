use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
};
use worth_store_physical_backend::FilesystemAccessPosture;

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn supported(root: std::path::PathBuf) {
    let admission = PhysicalRuntimeAdmission::new(root).unwrap();
    let runtime = PhysicalStore::admit(admission).unwrap();
    let media =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let _outcome = runtime.try_admit_filesystem_media(media);
}

pub(crate) fn main() {}
