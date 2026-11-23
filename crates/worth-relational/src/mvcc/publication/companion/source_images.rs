//! A published image belongs to one branch. A readmitted root can be shared.
use super::{CompanionPreflightStop as Stop, ManagedSourceRootImage};
use crate::history::data::BranchId;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex, OnceLock},
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone, Debug)]
pub(crate) struct ManagedSourceRootImages(Placement);

#[derive(Clone, Debug)]
enum Placement {
    Published(OnceLock<ManagedSourceRootImage>),
    Readmitted(OnceLock<Arc<ReadmittedImages>>),
}

#[derive(Debug)]
struct ReadmittedImages {
    branches: Mutex<ExecutionOrderedMap<BranchId, ManagedSourceRootImage>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl Default for ManagedSourceRootImages {
    fn default() -> Self {
        Self(Placement::Published(OnceLock::new()))
    }
}

impl ManagedSourceRootImages {
    pub(crate) fn readmitted() -> Self {
        Self(Placement::Readmitted(OnceLock::new()))
    }

    pub(crate) fn is_readmitted(&self) -> bool {
        matches!(self.0, Placement::Readmitted(_))
    }

    pub(crate) fn set(&self, image: ManagedSourceRootImage) -> Result<(), Stop> {
        let Placement::Published(published) = &self.0 else {
            return Err(Stop::SelectedSourceMismatch);
        };
        published
            .set(image)
            .map_err(|_| Stop::SelectedSourceMismatch)
    }

    pub(crate) fn for_branch(&self, branch: &BranchId) -> Option<Cow<'_, ManagedSourceRootImage>> {
        match &self.0 {
            Placement::Published(image) => image.get().map(Cow::Borrowed),
            Placement::Readmitted(images) => images
                .get()?
                .branches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(branch)
                .cloned()
                .map(Cow::Owned),
        }
    }

    /// The exclusive publication epoch excludes a competing branch publication
    /// or placement. The lookup callback only installs prepaid participant state.
    pub(super) fn attach_readmitted(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        branch: &BranchId,
        image: &ManagedSourceRootImage,
        install_lookup: impl FnOnce() -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        let Placement::Readmitted(readmitted) = &self.0 else {
            return Err(Stop::SelectedSourceMismatch);
        };
        if let Some(images) = readmitted.get() {
            let mut entries = images
                .branches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if entries.get(branch).is_some() {
                return Err(Stop::SelectedSourceMismatch);
            }
            let mut next = entries.clone();
            insert(execution, &mut next, branch, image)?;
            install_lookup()?;
            *entries = next;
            return Ok(());
        }
        let memory = execution
            .reserve_retained_memory(
                (size_of::<ReadmittedImages>() + 2 * size_of::<usize>()) as u64,
            )
            .map_err(|cause| Stop::Execution(cause.into()))?;
        let mut entries = ExecutionOrderedMap::default();
        insert(execution, &mut entries, branch, image)?;
        let images = Arc::new(ReadmittedImages {
            branches: Mutex::new(entries),
            _memory: memory,
        });
        install_lookup()?;
        readmitted
            .set(images)
            .expect("Native excludes competing attachment");
        Ok(())
    }
}

fn insert(
    execution: &ExecutionRequest<'_, '_>,
    entries: &mut ExecutionOrderedMap<BranchId, ManagedSourceRootImage>,
    branch: &BranchId,
    image: &ManagedSourceRootImage,
) -> Result<(), Stop> {
    entries
        .insert_cloned_native_for_request(branch, image, *execution)
        .map_err(crate::validation::candidate_source::managed_source_ordered_denial)
}
