use super::backed_path::{admit, unqualified};
use super::{
    ArtifactTreeAllocatedReadFailure as Failure, ArtifactTreeDirectory, ArtifactTreeFile,
    ArtifactTreeMedia, ArtifactTreePathAllocationBoundary as Boundary, ArtifactTreePathAllocator,
};
use cap_fs_ext::OpenOptionsFollowExt;
use cap_primitives::fs::FollowSymlinks;
use cap_std::fs::{Dir, File, OpenOptions};

impl ArtifactTreeMedia<'_> {
    pub(super) fn open_directory_with_allocator<A: ArtifactTreePathAllocator>(
        &self,
        directory: &ArtifactTreeDirectory,
        allocator: &mut A,
    ) -> Result<Dir, Failure<A::Denial>> {
        if !self.path_storage_is_qualified() {
            return Err(unqualified());
        }
        let Some((first, remaining)) = directory.components.split_first() else {
            return self.root(directory.root).try_clone().map_err(|error| {
                Failure::Media(super::ArtifactTreeFailure::io(
                    super::ArtifactTreeFailureKind::DeniedBeforeEffect,
                    &error,
                ))
            });
        };
        let mut current =
            self.open_component_with_allocator(self.root(directory.root), first, allocator)?;
        for component in remaining {
            current = self.open_component_with_allocator(&current, component, allocator)?;
        }
        Ok(current)
    }

    fn open_component_with_allocator<A: ArtifactTreePathAllocator>(
        &self,
        parent: &Dir,
        component: &str,
        allocator: &mut A,
    ) -> Result<Dir, Failure<A::Denial>> {
        let requested = super::path_storage::open_bytes(component).ok_or_else(unqualified)?;
        let backing = admit(allocator, Boundary::DirectoryOpen, requested)?;
        super::super::artifact_tree_effects::open_directory_with_backing(
            self.owner, parent, component, backing,
        )
        .map_err(Failure::Media)
    }

    pub(super) fn open_readable_file_with_allocator<A: ArtifactTreePathAllocator>(
        &self,
        directory: &Dir,
        artifact: &ArtifactTreeFile,
        allocator: &mut A,
    ) -> Result<File, Failure<A::Denial>> {
        let requested =
            super::path_storage::open_bytes(&artifact.file_name).ok_or_else(unqualified)?;
        let backing = admit(allocator, Boundary::FileOpen, requested)?;
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        self.open_file_with_backing(directory, &artifact.file_name, &options, backing)
            .map_err(Failure::Media)
    }

    pub(crate) fn directory_exists_with_allocator<A: ArtifactTreePathAllocator>(
        &self,
        directory: &ArtifactTreeDirectory,
        allocator: &mut A,
    ) -> Result<bool, Failure<A::Denial>> {
        match self.open_directory_with_allocator(directory, allocator) {
            Ok(_) => Ok(true),
            Err(Failure::Media(failure))
                if failure.kind() == super::ArtifactTreeFailureKind::Absent =>
            {
                Ok(false)
            }
            Err(failure) => Err(failure),
        }
    }
}
