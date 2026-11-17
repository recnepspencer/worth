//! The real synchronization handle must be flushable, including Linux O_PATH owners.
#[test]
fn capability_directory_opens_a_flushable_sync_handle() {
    let root = tempfile::tempdir().expect("temporary directory");
    let directory = cap_std::fs::Dir::open_ambient_dir(root.path(), cap_std::ambient_authority())
        .expect("directory capability");
    let handle = super::super::directory_synchronization::open_directory_sync_handle(&directory)
        .expect("readable sync handle under the capability");
    // Exercise the actual OS call even when the test build skips ordinary flushes.
    handle
        .sync_all()
        .expect("directory publication can be synchronized");
}
