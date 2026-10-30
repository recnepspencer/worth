use worth_store::physical_runtime::{
    execute_recovery_cleanup_removal, AdmittedRecoveryFilesystemMedia,
};

fn supplied<T>() -> T {
    panic!("compile-only supplied production value")
}

fn bypass(media: &AdmittedRecoveryFilesystemMedia) {
    // Even a caller that possesses genuine lower-layer values cannot invoke a
    // cleanup effect from the Store facade's admitted-media handle. Cleanup
    // remains a private Store coordination continuation.
    let _ = media.remove_revalidated_recovery_artifact_scheduled(
        supplied(),
        supplied(),
        supplied(),
    );
    let _ = execute_recovery_cleanup_removal(media, supplied(), supplied());
}

fn main() {
    bypass(supplied());
}
