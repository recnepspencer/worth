//! Fragment-facing code is independent of the installed schema's Rust type.

mod access;
#[cfg(test)]
mod declaration_contract;
mod denial;
mod entity;
mod read;
mod write;

pub(in crate::domain_computation::primary_graph) use access::{ReadPort, WritePort};
pub use access::{Reading, Writing};
pub use denial::FragmentDenial;
pub use entity::{CandidateEntity, ObservedEntity, ObservedTarget};
pub(in crate::domain_computation::primary_graph) use entity::{ErasedCandidate, ErasedObserved};
#[doc(hidden)]
pub use read::Readable;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{Fragment, FragmentDefinition};

/// The access loan preserves Query's decision/read and candidate/write phases.
/// The schema is checked at acquisition and does not parameterize this view.
pub struct View<'a, F: Fragment, Access> {
    pub(super) definition: &'a FragmentDefinition,
    pub(super) access: Access,
    marker: PhantomData<fn() -> F>,
}

impl<'a, F: Fragment> View<'a, F, Reading<'a>> {
    pub(in crate::domain_computation::primary_graph) fn reading(
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
    pub(in crate::domain_computation::primary_graph) fn writing(
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
