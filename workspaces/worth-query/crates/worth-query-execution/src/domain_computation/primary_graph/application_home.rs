//! The home one application opens on and closes back into.
//!
//! A home is a value. An open consumes it and a refused open hands it back by
//! phase, so a failed open never loses it. It is deliberately not `Clone`: two
//! runtimes opened from one image would be two diverging histories of one home.

mod close;
mod close_refusal;
mod deferral;
pub use close_refusal::{WorthQueryApplicationCloseDenial, WorthQueryApplicationCloseRefusal};
#[cfg(feature = "test-durability-faults")]
mod durability_seam;
#[cfg(test)]
mod lifecycle_fixture;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use super::WorthQueryApplicationCheckpoint;
pub use deferral::{
    WorthQueryHomeAbsent, WorthQueryHomeForm, WorthQueryReopenDeferral, WorthQueryReturnPoint,
    WorthQueryStateOwner,
};

/// Where one application's state lives between opens.
pub struct ApplicationHome(HomeForm);

enum HomeForm {
    /// Process memory. It holds the closed image once a runtime closed into it.
    Memory(Option<WorthQueryApplicationCheckpoint>),
    /// A durable location. No open reads it until Relational runs on Store.
    At(PathBuf),
}

impl std::fmt::Debug for ApplicationHome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            HomeForm::Memory(image) => formatter
                .debug_struct("ApplicationHome::Memory")
                .field("holds_image", &image.is_some())
                .finish(),
            HomeForm::At(path) => formatter
                .debug_tuple("ApplicationHome::At")
                .field(path)
                .finish(),
        }
    }
}

impl ApplicationHome {
    /// An empty home in process memory.
    pub const fn memory() -> Self {
        Self(HomeForm::Memory(None))
    }

    /// A home at a durable location. Naming it touches no filesystem.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self(HomeForm::At(path.into()))
    }

    /// The memory home an image settles into.
    pub(in crate::domain_computation::primary_graph) const fn holding(
        image: WorthQueryApplicationCheckpoint,
    ) -> Self {
        Self(HomeForm::Memory(Some(image)))
    }

    /// The image an open resumes, or `None` for an empty home. A form no open
    /// can read yet answers with its deferral before any filesystem call.
    pub(in crate::domain_computation::primary_graph) fn closed_image(
        &self,
    ) -> Result<Option<&WorthQueryApplicationCheckpoint>, WorthQueryHomeAbsent> {
        match &self.0 {
            HomeForm::Memory(image) => Ok(image.as_ref()),
            HomeForm::At(_) => Err(WorthQueryHomeAbsent {
                form: WorthQueryHomeForm::At,
                deferral: WorthQueryReopenDeferral {
                    owner: WorthQueryStateOwner::Relational,
                    return_point: WorthQueryReturnPoint::RelationalOnStore,
                },
            }),
        }
    }
}
