use super::{
    ArtifactTreeAllocatedReadFailure as Failure, ArtifactTreeDirectory, ArtifactTreeFile,
    ArtifactTreePathAllocationBoundary as Boundary, ArtifactTreePathAllocator,
};

/// Data is disposed before the backing which admitted its construction.
pub(crate) struct ArtifactTreeBackedPath<T, B> {
    pub(super) data: T,
    _backing: B,
}

impl<T, B> ArtifactTreeBackedPath<T, B> {
    pub(crate) fn get(&self) -> &T {
        &self.data
    }
}

pub(crate) fn backed_directory<A: ArtifactTreePathAllocator>(
    parent: &ArtifactTreeDirectory,
    component: &str,
    allocator: &mut A,
) -> Result<ArtifactTreeBackedPath<ArtifactTreeDirectory, A::PathBacking>, Failure<A::Denial>> {
    let requested = super::path_storage::child_bytes(parent, component).ok_or_else(unqualified)?;
    let backing = admit(allocator, Boundary::DirectoryAddress, requested)?;
    let data = parent.child(component).map_err(|_| unqualified())?;
    Ok(ArtifactTreeBackedPath {
        data,
        _backing: backing,
    })
}

pub(crate) fn backed_file<A: ArtifactTreePathAllocator>(
    parent: &ArtifactTreeDirectory,
    component: &str,
    allocator: &mut A,
) -> Result<ArtifactTreeBackedPath<ArtifactTreeFile, A::PathBacking>, Failure<A::Denial>> {
    let requested = super::path_storage::file_bytes(parent, component).ok_or_else(unqualified)?;
    let backing = admit(allocator, Boundary::FileAddress, requested)?;
    let data = parent.file(component).map_err(|_| unqualified())?;
    Ok(ArtifactTreeBackedPath {
        data,
        _backing: backing,
    })
}

pub(super) fn admit<A: ArtifactTreePathAllocator>(
    allocator: &mut A,
    boundary: Boundary,
    requested: u64,
) -> Result<A::PathBacking, Failure<A::Denial>> {
    let requested = usize::try_from(requested).map_err(|_| unqualified())?;
    allocator
        .admit_path_backing(boundary, requested as u64)
        .map_err(|cause| Failure::Allocation { requested, cause })
}

pub(super) fn unqualified<E>() -> Failure<E> {
    Failure::Media(super::ArtifactTreeFailure::structural(
        super::ArtifactTreeFailureKind::AccessLimitExceeded,
    ))
}
