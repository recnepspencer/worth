//! A cutover image belongs to its publishing branch. A readmitted root, or a
//! live head its registration left vacant, can be shared by several branches,
//! each with its own image.
use super::ManagedSourceRootImage;
use crate::history::data::BranchId;
use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop as Stop;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex, OnceLock},
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone, Debug)]
pub struct ManagedSourceRootImages(Placement);

#[derive(Clone, Debug)]
enum Placement {
    Published(OnceLock<PublishedSlot>),
    Readmitted(OnceLock<Arc<BranchImages>>),
}

/// What a published root carries once a Source image is placed on it.
#[derive(Clone, Debug)]
enum PublishedSlot {
    /// Set by publication cutover before the root went live: the publishing
    /// branch's image, so any other branch sees that registration's image.
    Cutover(ManagedSourceRootImage),
    /// Attached to a live head its publishing registration left vacant.
    /// Branches forked from that head share the root, so each attaches its
    /// own image.
    Vacant(Arc<BranchImages>),
}

/// Per-branch images of a root several live branches share.
#[derive(Debug)]
struct BranchImages {
    branches: Mutex<ExecutionOrderedMap<BranchId, ManagedSourceRootImage>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl Default for ManagedSourceRootImages {
    fn default() -> Self {
        Self(Placement::Published(OnceLock::new()))
    }
}

impl ManagedSourceRootImages {
    pub fn readmitted() -> Self {
        Self(Placement::Readmitted(OnceLock::new()))
    }

    pub fn is_readmitted(&self) -> bool {
        matches!(self.0, Placement::Readmitted(_))
    }

    pub fn set(&self, image: ManagedSourceRootImage) -> Result<(), Stop> {
        let Placement::Published(published) = &self.0 else {
            return Err(Stop::SelectedSourceMismatch);
        };
        published
            .set(PublishedSlot::Cutover(image))
            .map_err(|_| Stop::SelectedSourceMismatch)
    }

    pub fn for_branch(&self, branch: &BranchId) -> Option<Cow<'_, ManagedSourceRootImage>> {
        let images = match &self.0 {
            Placement::Published(slot) => match slot.get()? {
                PublishedSlot::Cutover(image) => return Some(Cow::Borrowed(image)),
                PublishedSlot::Vacant(images) => images,
            },
            Placement::Readmitted(images) => images.get()?,
        };
        images
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(branch)
            .cloned()
            .map(Cow::Owned)
    }

    /// A live head that carries no Source image for `branch`: readmitted, or
    /// published while its registration placed none (such as a Home program
    /// transition step committed by the outgoing registration). Under the
    /// same exclusion as a readmitted attachment, the active registration
    /// places one image for `branch`; branches sharing the head attach their
    /// own. Publication sets a cutover image only on an unpublished root, so a
    /// vacant live head is never also set by cutover.
    pub fn attach_vacant(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        branch: &BranchId,
        image: &ManagedSourceRootImage,
        install_lookup: impl FnOnce() -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        match &self.0 {
            Placement::Readmitted(readmitted) => attach_branch(
                execution,
                readmitted.get(),
                |images| readmitted.set(images).map_err(|_| ()),
                (branch, image),
                install_lookup,
            ),
            Placement::Published(published) => {
                let existing = match published.get() {
                    Some(PublishedSlot::Cutover(_)) => return Err(Stop::SelectedSourceMismatch),
                    Some(PublishedSlot::Vacant(images)) => Some(images),
                    None => None,
                };
                attach_branch(
                    execution,
                    existing,
                    |images| published.set(PublishedSlot::Vacant(images)).map_err(|_| ()),
                    (branch, image),
                    install_lookup,
                )
            }
        }
    }
}

/// Adds `branch`'s image to a shared root's per-branch images, creating them
/// on first attachment. The exclusive publication epoch excludes a competing
/// branch publication or placement. The lookup callback only installs
/// prepaid participant state.
fn attach_branch(
    execution: &ExecutionRequest<'_, '_>,
    existing: Option<&Arc<BranchImages>>,
    initialize: impl FnOnce(Arc<BranchImages>) -> Result<(), ()>,
    (branch, image): (&BranchId, &ManagedSourceRootImage),
    install_lookup: impl FnOnce() -> Result<(), Stop>,
) -> Result<(), Stop> {
    if let Some(images) = existing {
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
        .reserve_retained_memory((size_of::<BranchImages>() + 2 * size_of::<usize>()) as u64)
        .map_err(|cause| Stop::Execution(cause.into()))?;
    let mut entries = ExecutionOrderedMap::default();
    insert(execution, &mut entries, branch, image)?;
    let images = Arc::new(BranchImages {
        branches: Mutex::new(entries),
        _memory: memory,
    });
    install_lookup()?;
    // The head exclusion keeps the cell empty since `existing` was read; a
    // refused set means that exclusion was bypassed, and refuses typed.
    initialize(images).map_err(|()| Stop::SelectedSourceMismatch)
}

fn insert(
    execution: &ExecutionRequest<'_, '_>,
    entries: &mut ExecutionOrderedMap<BranchId, ManagedSourceRootImage>,
    branch: &BranchId,
    image: &ManagedSourceRootImage,
) -> Result<(), Stop> {
    entries
        .insert_cloned_native_for_request(branch, image, *execution)
        .map_err(crate::validation::candidate_source::image::entry::ordered_denial)
}
