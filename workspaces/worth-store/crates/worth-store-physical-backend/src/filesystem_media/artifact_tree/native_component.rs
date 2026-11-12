//! One validated component, opened relative to the held directory without links.
use cap_std::fs::Dir;
use rustix::fs::{Mode, OFlags};
use std::{ffi::CString, io};

use super::media_open::FileAccess;

fn name(component: &str) -> io::Result<CString> {
    super::path::validate_component(component).map_err(|_| io::ErrorKind::InvalidInput)?;
    let capacity = component
        .len()
        .checked_add(1)
        .ok_or(io::ErrorKind::InvalidInput)?;
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(component.as_bytes());
    bytes.push(0);
    CString::from_vec_with_nul(bytes).map_err(|_| io::ErrorKind::InvalidInput.into())
}

pub(in crate::filesystem_media) fn directory(parent: &Dir, component: &str) -> io::Result<Dir> {
    let name = name(component)?;
    let fd = rustix::fs::openat(
        parent,
        &name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )?;
    Ok(Dir::from_std_file(std::fs::File::from(fd)))
}

pub(super) fn file(
    parent: &Dir,
    component: &str,
    access: FileAccess,
) -> io::Result<cap_std::fs::File> {
    let name = name(component)?;
    let access = match access {
        FileAccess::ReadOnly => OFlags::RDONLY,
        FileAccess::ReadWrite => OFlags::RDWR,
    };
    let fd = rustix::fs::openat(
        parent,
        &name,
        access | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )?;
    Ok(cap_std::fs::File::from_std(std::fs::File::from(fd)))
}
