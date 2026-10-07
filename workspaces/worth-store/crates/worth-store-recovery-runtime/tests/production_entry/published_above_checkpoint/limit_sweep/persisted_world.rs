use std::path::Path;

use super::worlds::Killed;

impl Killed for tempfile::TempDir {
    fn root(&self) -> &Path {
        self.path()
    }
}

/// Copy the exact media left by the genuine killed producer. Each sweep may
/// recover its own copy without rebuilding the producer or changing its peers.
pub(super) fn copy_killed_media(source: &Path) -> tempfile::TempDir {
    let parent = source.parent().expect("killed root has a fixture parent");
    let destination = tempfile::Builder::new()
        .prefix("killed-media-copy-")
        .tempdir_in(parent)
        .expect("independent killed-media copy");
    copy_directory(source, destination.path());
    destination
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create killed-media directory");
    for entry in std::fs::read_dir(source).expect("read killed-media source") {
        let entry = entry.expect("read killed-media artifact");
        let target = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("killed-media artifact type")
            .is_dir()
        {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy genuine killed-media artifact");
        }
    }
}
