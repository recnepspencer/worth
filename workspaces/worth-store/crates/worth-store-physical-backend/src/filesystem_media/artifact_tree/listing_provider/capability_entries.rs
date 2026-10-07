use cap_std::fs::{Dir, DirEntry, ReadDir};
use std::io;

pub(in crate::filesystem_media::artifact_tree) struct DirectoryEntries {
    entries: ReadDir,
    _directory: Dir,
}

pub(in crate::filesystem_media::artifact_tree) fn open(
    directory: Dir,
) -> io::Result<DirectoryEntries> {
    let entries = directory.entries()?;
    Ok(DirectoryEntries {
        entries,
        _directory: directory,
    })
}

impl Iterator for DirectoryEntries {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}
