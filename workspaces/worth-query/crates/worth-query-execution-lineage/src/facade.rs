//! Crate facade for primary-graph lineage, output demand, invariant projection,
//! partitioned computation, handlers, checkpoints, and domain semantics.

pub mod domain_computation {
    pub use crate::domain_computation::{
        application_aftermath, authorization, execution_observation, primary_graph,
        product_publication,
    };
}
pub use crate::domain_computation::{application_aftermath, authorization, primary_graph};
pub use worth_query_execution_obligations::facade::aftermath::correction_material;
pub mod ports {
    pub use crate::domain_computation::authorization::application_disclosure::receipt::ApplicationDisclosureReceiptInternalPort;
    pub use crate::domain_computation::primary_graph::application_attempt::compare_and_commit::cleanup_required::internal_port::ApplicationCleanupRequiredInternalPort;
    pub use crate::domain_computation::primary_graph::application_attempt::compare_and_commit::committed_publication::readmitted::CommittedProductPublicationInternalPort;
    pub use crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::family_entry::ApplicationOutputFamilyEntryInternalPort;
    pub use crate::domain_computation::primary_graph::application_checkpoint::internal_port::{
        ApplicationCheckpointInternalPort, InterimCheckpointOriginInternalPort,
    };
    pub use crate::domain_computation::primary_graph::application_contribution::computation::checkpoint::{
        ManagedComputationCheckpointInternalPort, ManagedComputationExecutionInternalPort,
    };
    pub use crate::domain_computation::primary_graph::application_contribution::conditional::package_contract::ConditionalPackageContractInternalPort;
    pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::plan::{
        ComputationPartitionMembersInternalPort, ComputationPartitionPlanInternalPort,
        ComputationPartitionViewInternalPort,
    };
    pub use crate::domain_computation::primary_graph::application_contribution::producer::demand::denial::internal_port::OutputDemandDenialInternalPort;
    pub use crate::domain_computation::primary_graph::application_program::PreparedRequiredOutputSourceInternalPort;
    pub use crate::domain_computation::primary_graph::application_query::observed_source::denial::SourceExpectationDenialInternalPort;
    pub use crate::domain_computation::primary_graph::bootstrap::publication::PrimaryGraphPublicationInternalPort;
    pub use crate::domain_computation::primary_graph::fragment_view::entity::{
        FragmentCandidateEntityInternalPort, FragmentObservedEntityInternalPort,
        FragmentObservedTargetInternalPort,
    };
    pub use crate::domain_computation::primary_graph::fragment_view::{
        FragmentReadingViewInternalPort, FragmentWritingViewInternalPort,
    };
    pub use crate::domain_computation::primary_graph::invariant_projection::internal_port::{
        ApplicationInvariantProjectionAuthorityInternalPort,
        ApplicationInvariantProjectionExecutionInternalPort,
        ApplicationInvariantProjectionSnapshotInternalPort,
    };
    pub use crate::domain_computation::primary_graph::invariant_projection::operation_reader::current_output::CurrentOutputDenialInternalPort;
    pub use crate::domain_computation::primary_graph::output_lineage::internal_port::ApplicationOutputLineageInternalPort;
    pub use crate::domain_computation::primary_graph::root::integration_handle_port::PrimaryGraphIntegrationHandleInternalPort;
    pub use crate::domain_computation::primary_graph::root::internal_port::PrimaryGraphInternalPort;
    pub use crate::domain_computation::product_publication::unpublished_internal_port::ProductUnpublishedApplicationInternalPort;
    pub use crate::ports::*;
}
pub mod test_read_stops {
    pub use crate::test_read_stops::*;
}
