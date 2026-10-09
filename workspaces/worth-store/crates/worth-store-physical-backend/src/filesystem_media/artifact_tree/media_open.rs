use cap_fs_ext::OpenOptionsFollowExt;
use cap_primitives::fs::FollowSymlinks;
use cap_std::fs::{Dir, OpenOptions};

use super::super::artifact_tree_effects::{begin, open_directory, open_optional_directory};
use super::path::ArtifactTreeRoot;
use super::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeMedia,
};
use crate::filesystem_media::MediaOperationRole;

impl ArtifactTreeMedia<'_> {
    pub(super) fn open_mutable_file(
        &self,
        directory: &Dir,
        file_name: &str,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).follow(FollowSymlinks::No);
        self.open_file(directory, file_name, &options)
    }

    pub(super) fn open_readable_file(
        &self,
        directory: &Dir,
        file_name: &str,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        self.open_file(directory, file_name, &options)
    }

    fn open_file(
        &self,
        directory: &Dir,
        file_name: &str,
        options: &OpenOptions,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        self.open_file_with_backing(directory, file_name, options, ())
    }

    pub(super) fn open_file_with_backing<B>(
        &self,
        directory: &Dir,
        file_name: &str,
        options: &OpenOptions,
        backing: B,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        let open = begin(self.owner, MediaOperationRole::OpenExisting, 0);
        if let Some(error) = open.fail_before_error() {
            drop(backing);
            open.denied();
            return Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::DeniedBeforeEffect,
                &error,
            ));
        }
        let result = directory.open_with(file_name, options);
        drop(backing);
        match result {
            Ok(file) => {
                open.completed(0);
                Ok(file)
            }
            Err(error) => {
                open.denied();
                Err(ArtifactTreeFailure::io(
                    if error.kind() == std::io::ErrorKind::NotFound {
                        ArtifactTreeFailureKind::Absent
                    } else {
                        ArtifactTreeFailureKind::DeniedBeforeEffect
                    },
                    &error,
                ))
            }
        }
    }

    pub(super) fn open_optional_directory(
        &self,
        directory: &ArtifactTreeDirectory,
    ) -> Result<Option<Dir>, ArtifactTreeFailure> {
        let Some((first, remaining)) = directory.components.split_first() else {
            return self
                .root(directory.root)
                .try_clone()
                .map(Some)
                .map_err(|error| {
                    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
                });
        };
        let Some(mut current) =
            open_optional_directory(self.owner, self.root(directory.root), first)?
        else {
            return Ok(None);
        };
        for component in remaining {
            let Some(next) = open_optional_directory(self.owner, &current, component)? else {
                return Ok(None);
            };
            current = next;
        }
        Ok(Some(current))
    }

    pub(super) fn open_directory(
        &self,
        directory: &ArtifactTreeDirectory,
    ) -> Result<Dir, ArtifactTreeFailure> {
        self.open_components(directory.root, &directory.components)
    }

    pub(super) fn open_components(
        &self,
        root: ArtifactTreeRoot,
        components: &[String],
    ) -> Result<Dir, ArtifactTreeFailure> {
        let Some((first, remaining)) = components.split_first() else {
            return self.root(root).try_clone().map_err(|error| {
                ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
            });
        };
        let mut current = open_directory(self.owner, self.root(root), first)?;
        for component in remaining {
            current = open_directory(self.owner, &current, component)?;
        }
        Ok(current)
    }

    pub(super) fn root(&self, root: ArtifactTreeRoot) -> &Dir {
        match root {
            ArtifactTreeRoot::Families => self.owner.families().handle().directory(),
            ArtifactTreeRoot::Staging => self.owner.staging().handle().directory(),
        }
    }
}
