//! The runtime's consumer methods over `domain_computation::primary_graph::application_checkpoint`; each only calls its subsystem.
use crate::facade::application_installation::WorthQueryApplicationCheckpoint;
use crate::facade::application_installation::WorthQueryApplicationCheckpointSectionBytes;
use crate::facade::application_installation::WorthQueryCheckpointCaptureDenial;
use crate::facade::primary_graph::WorthQueryApplicationCheckpointInspection;
use crate::facade::primary_graph::WorthQueryCheckpointInspectionDenial;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn capture_application_checkpoint(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        policy: crate::domain_computation::primary_graph::application_checkpoint::WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial> {
        <Self as crate::domain_computation::primary_graph::application_checkpoint::CaptureOps<
            Schema,
        >>::capture_application_checkpoint(self, phase, policy)
    }

    pub fn capture_application_checkpoint_with_sections(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        policy: crate::domain_computation::primary_graph::application_checkpoint::WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (
            WorthQueryApplicationCheckpoint,
            WorthQueryApplicationCheckpointSectionBytes,
        ),
        WorthQueryCheckpointCaptureDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_checkpoint::CaptureOps<
            Schema,
        >>::capture_application_checkpoint_with_sections(self, phase, policy)
    }

    pub fn inspect_application_checkpoint<'a>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        bytes: &'a [u8],
    ) -> Result<WorthQueryApplicationCheckpointInspection<'a>, WorthQueryCheckpointInspectionDenial>
    {
        <Self as crate::domain_computation::primary_graph::application_checkpoint::InspectionOps<
            Schema,
        >>::inspect_application_checkpoint(self, phase, bytes)
    }
}
