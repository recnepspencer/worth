//! Recorded and requested meaning remain distinct on a refused opening.
use super::super::refusal::WorthQueryApplicationHomeOpenRefused;
use worth_foundational::facade::ApplicationHomeBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_store_application::{ApplicationHomeDescriptorDenial, ApplicationHomeOpenRefusal};

/// Descriptive revisions from the actual descriptor refusal. This value neither
/// admits a predecessor nor permits transition execution.
#[derive(Debug)]
pub struct WorthQueryApplicationHomeTransitionRequired<'refusal> {
    recorded: &'refusal ApplicationHomeBinding,
    requested: &'refusal ApplicationHomeBinding,
}
impl WorthQueryApplicationHomeTransitionRequired<'_> {
    pub fn recorded(&self) -> &ApplicationHomeBinding {
        self.recorded
    }
    pub fn requested(&self) -> &ApplicationHomeBinding {
        self.requested
    }
}

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Inspect a transition requirement without importing the Store composition
    /// facade or interpreting a diagnostic string.
    pub fn transition_required(&self) -> Option<WorthQueryApplicationHomeTransitionRequired<'_>> {
        let Self::Physical {
            cause:
                ApplicationHomeOpenRefusal::Descriptor(
                    ApplicationHomeDescriptorDenial::TransitionRequired {
                        recorded,
                        requested,
                        ..
                    },
                ),
            ..
        } = self
        else {
            return None;
        };
        Some(WorthQueryApplicationHomeTransitionRequired {
            recorded,
            requested,
        })
    }
}
