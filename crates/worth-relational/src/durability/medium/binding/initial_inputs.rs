//! Initialization uses the same subject-bound publication fate and barrier.
use super::*;
use crate::durability::source_initial_inputs::image::InitialInputsImage;
use crate::durability::source_initial_inputs::RelationalInitialInputsPublicationDetail as Detail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalInitialInputsPublicationStop {
    Request(crate::execution::RelationalExecutionDenialCause),
    Refused(RelationalDurablePublicationRefusal),
    Unresolved(RelationalDurabilityUnresolved),
}
#[derive(Debug)]
pub(crate) struct RelationalCommittedInitialInputs {
    pub(crate) image: Arc<InitialInputsImage>,
}
impl AuthorityMarker for RelationalCommittedInitialInputs {}
impl RelationalDurableMediumBinding {
    pub(crate) fn publish_initial_inputs(
        &self,
        image: Arc<InitialInputsImage>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        DurablyCommitted<RelationalCommittedInitialInputs>,
        RelationalInitialInputsPublicationStop,
    > {
        let artifact = image
            .artifact
            .as_ref()
            .expect("producer initialization retains its original frame");
        let subject = RelationalDurabilityUnresolvedSubject::InitialInputs {
            revision: artifact.revision(),
            digest: artifact.payload_digest(),
        };
        let state = self.state();
        let medium = state.admitting_medium().map_err(|cause| match cause {
            RelationalDurableStepStop::Refused(cause) => {
                RelationalInitialInputsPublicationStop::Refused(cause)
            }
            RelationalDurableStepStop::Unresolved(cause) => {
                RelationalInitialInputsPublicationStop::Unresolved(cause)
            }
        })?;
        match medium.publish_initial_inputs(artifact, request) {
            DurablePublicationFate::Committed => Ok(DurablyCommitted::from_owner(
                RelationalCommittedInitialInputs { image },
            )),
            DurablePublicationFate::ProvenNoEffect {
                detail: Detail::Request(cause),
            } => Err(RelationalInitialInputsPublicationStop::Request(cause)),
            DurablePublicationFate::ProvenNoEffect { detail } => {
                Err(RelationalInitialInputsPublicationStop::Refused(
                    RelationalDurablePublicationRefusal::ProvenNoEffect {
                        detail: format!("{detail:?}"),
                    },
                ))
            }
            DurablePublicationFate::Indeterminate { detail } => {
                Err(RelationalInitialInputsPublicationStop::Unresolved(
                    state.record_unresolved(subject, format!("{detail:?}")),
                ))
            }
        }
    }
}
