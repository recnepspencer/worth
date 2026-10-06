//! The typed plan both open builders lower into.
//!
//! The plan names the entry and how the home starts, so an initial state
//! without an empty home, or an adoption without an image, cannot be spelled.

use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::program_admission::WorthQueryProgramAdmissionStep;
use super::WorthQueryOpenAdoption;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCheckpoint, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource;

/// The initial state an open declares. It runs only when the home starts empty.
pub(in crate::domain_computation::primary_graph) type InitialState<'open, Schema> = Box<
    dyn FnOnce(
            &mut WorthQueryPrimaryGraphBootstrap<Schema>,
            &WorthQueryInstalledApplicationSchema<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
        + 'open,
>;

pub(in crate::domain_computation::primary_graph) struct OpenPlan<'open, Schema> {
    pub(in crate::domain_computation::primary_graph) entry: OpenEntry<'open, Schema>,
    pub(in crate::domain_computation::primary_graph) start: HomeStart<'open, Schema>,
    pub(in crate::domain_computation::primary_graph) authorization_time_source:
        Option<Box<dyn WorthQueryRuntimeTimeSource>>,
}

pub(in crate::domain_computation::primary_graph) enum OpenEntry<'open, Schema> {
    /// Contribution-owned schema meaning without an application program.
    Declaration,
    /// A program entry, admitted against its own installed schema.
    Program(WorthQueryProgramAdmissionStep<'open, Schema>),
}

pub(in crate::domain_computation::primary_graph) enum HomeStart<'open, Schema> {
    Empty {
        initial_state: InitialState<'open, Schema>,
    },
    Resume {
        image: WorthQueryApplicationCheckpoint,
        adoption: Option<WorthQueryOpenAdoption<'open, Schema>>,
    },
}

impl<'open, Schema> HomeStart<'open, Schema> {
    /// The start an open performs on a home holding `image`, or on an empty one.
    pub(in crate::domain_computation::primary_graph) fn of(
        image: Option<&WorthQueryApplicationCheckpoint>,
        initial_state: InitialState<'open, Schema>,
        adoption: Option<WorthQueryOpenAdoption<'open, Schema>>,
    ) -> Self {
        match image {
            // Decoding consumes the image, so open reads a copy and the given
            // home stays whole for an unchanged refusal.
            Some(image) => Self::Resume {
                image: image.clone(),
                adoption,
            },
            None => Self::Empty { initial_state },
        }
    }
}
