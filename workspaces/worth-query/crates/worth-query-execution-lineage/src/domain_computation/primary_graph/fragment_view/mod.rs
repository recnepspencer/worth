//! Fragment-facing code is independent of the installed schema's Rust type.

pub mod access;
#[cfg(test)]
mod declaration_contract;
pub mod denial;
pub mod entity;
pub mod group_read;
pub mod read;
pub mod relation;
pub mod write;

pub use access::{ReadPort, WritePort};
pub use access::{Reading, Writing};
pub use denial::FragmentDenial;
pub use entity::{CandidateEntity, ObservedEntity, ObservedTarget};
pub use entity::{ErasedCandidate, ErasedObserved};
#[doc(hidden)]
pub use read::Readable;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{Fragment, FragmentDefinition};

/// The access loan preserves Query's decision/read and candidate/write phases.
/// The schema is checked at acquisition and does not parameterize this view.
pub struct View<'a, F: Fragment, Access> {
    pub(crate) definition: &'a FragmentDefinition,
    pub(crate) access: Access,
    marker: PhantomData<fn() -> F>,
}

impl<'a, F: Fragment> View<'a, F, Reading<'a>> {
    pub(crate) fn reading(
        definition: &'a FragmentDefinition,
        access: &'a mut dyn ReadPort,
    ) -> Self {
        Self {
            definition,
            access: Reading { port: access },
            marker: PhantomData,
        }
    }
}
impl<'a, F: Fragment> View<'a, F, Writing<'a>> {
    pub(crate) fn writing(
        definition: &'a FragmentDefinition,
        access: &'a mut dyn WritePort,
    ) -> Self {
        Self {
            definition,
            access: Writing { port: access },
            marker: PhantomData,
        }
    }
}

/// Crate-boundary internal port for constructing fragment read/write views.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait FragmentReadingViewInternalPort<'a, F: Fragment>: Sized {
    fn from_reading(definition: &'a FragmentDefinition, access: &'a mut dyn ReadPort) -> Self;
}

impl<'a, F: Fragment> FragmentReadingViewInternalPort<'a, F> for View<'a, F, Reading<'a>> {
    fn from_reading(definition: &'a FragmentDefinition, access: &'a mut dyn ReadPort) -> Self {
        Self::reading(definition, access)
    }
}

pub trait FragmentWritingViewInternalPort<'a, F: Fragment>: Sized {
    fn from_writing(definition: &'a FragmentDefinition, access: &'a mut dyn WritePort) -> Self;
}

impl<'a, F: Fragment> FragmentWritingViewInternalPort<'a, F> for View<'a, F, Writing<'a>> {
    fn from_writing(definition: &'a FragmentDefinition, access: &'a mut dyn WritePort) -> Self {
        Self::writing(definition, access)
    }
}
