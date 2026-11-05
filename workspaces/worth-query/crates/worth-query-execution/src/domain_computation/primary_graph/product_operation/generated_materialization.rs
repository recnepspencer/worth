use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::{
    RelationalMaterializationCustody, RelationalMaterializationRecord,
};

mod reconstruction;
pub use reconstruction::{
    WorthQueryCompletedGeneratedOutputReconstruction, WorthQueryGeneratedEntity,
    WorthQueryGeneratedOutputReconstruction, WorthQueryGeneratedOutputReconstructionDenial,
    WorthQueryGeneratedOutputReconstructionFailure, WorthQueryReconstructedOutputEntity,
    WorthQueryRetainedGeneratedOutputEntity,
};
mod suspension;
pub use suspension::{
    WorthQueryGeneratedOutputSuspensionRecovery,
    WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    WorthQueryGeneratedOutputSuspensionRecoveryStage,
};
mod preparation_execution;
pub(crate) mod restoration;
mod suspend;
mod suspension_failure;

use suspend::suspended_output;
pub use {
    restoration::WorthQueryGeneratedOutputPublicationNoEffect,
    restoration::WorthQueryGeneratedOutputPublicationNoEffectCause,
    restoration::WorthQueryGeneratedOutputRestorationFailure,
    restoration::WorthQueryGeneratedOutputRestorationFailureCause,
    restoration::WorthQueryGeneratedOutputRestorationReceipt,
    restoration::WorthQueryGeneratedOutputRestorationRecovery,
    restoration::WorthQueryGeneratedOutputRestorationRecoveryFailure,
    restoration::WorthQueryGeneratedOutputRestorationRecoveryStage,
    restoration::WorthQueryRestoredGeneratedOutput,
    restoration::WorthQueryUnpublishedGeneratedOutputRestoration,
};

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider,
};

pub struct ProducerQualification {

    binding_type: TypeId,
    output_binding_type: TypeId,
    binding_identity: &'static str,
    provider_identity: &'static str,
    runtime_source_identity:
        crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity,
    checkpoint_source_identity:
        crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity,
    recorded_source_identity:
        crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity,
    source_partition_identity: [u8; 32],
    producer_dependency_identity: Option<[u8; 32]>,
    producing_implementation: Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    idempotency_key_identity: [u8; 32],
    runtime_authority: u64,
    schema: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    output_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    output_generation: u64,
    observed_source_facts:
        crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts,
}

/// A generated output taken out of its product branch and held in custody, so
/// it can be reconstructed and restored.
///
/// Returned by `suspend_current_generated_output` once the suspension is
/// published. Pass it to `reconstruct_generated_output`, then restore the
/// completed reconstruction with `restore_generated_output`. It is the only
/// custody of the suspended output.
#[must_use = "suspended output custody is required to restore the selected product"]
pub struct WorthQuerySuspendedGeneratedOutput {
    pub(super) publication: WorthQueryProductPublicationBinding,
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) custody: RelationalMaterializationCustody,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: ProducerQualification,
}

impl WorthQuerySuspendedGeneratedOutput {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub(super) fn custody_records(&self) -> &[RelationalMaterializationRecord] {
        self.custody.records()
    }

    pub(super) fn correspondence(&self) -> &WorthQueryApplicationOutputCorrespondence {
        &self.correspondence
    }

    pub(super) fn matches_producer_binding<Schema, Producer>(&self) -> bool
    where
        Schema: ApplicationSchema,
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.producer.binding_type == TypeId::of::<Producer>()
            && self.producer.binding_identity == Producer::IDENTITY
    }

    // In-memory custody sees the same compiled constant. Keeping this check at
    // the custody boundary also protects a future restart-restored token.
    pub(super) fn matches_provider_version<Schema, Producer>(&self) -> bool
    where
        Schema: ApplicationSchema,
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.producer.provider_identity == Producer::Provider::SEMANTIC_IDENTITY
    }

    pub(super) fn matches_runtime<Schema>(
        &self,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>,
    ) -> bool
    where
        Schema: ApplicationSchema,
    {
        self.producer.runtime_authority == runtime.authority_identity().as_u64()
            && self.producer.schema == runtime.installed_schema().binding_identity()
    }

    pub(super) fn matches_retained_lineage<Schema, Producer>(
        &self,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>,
    ) -> bool
    where
        Schema: ApplicationSchema,
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        runtime
            .provider_graph()
            .output_lineage
            .lock()
            .expect("application output lineage lock is available")
            .qualified_output::<Producer::Operation>(
                self.producer.runtime_authority,
                &self.producer.schema,
                self.producer.scope,
                self.producer.output_occurrence,
                self.producer.output_generation,
                self.producer.runtime_source_identity,
                self.producer.checkpoint_source_identity,
            )
            .is_some_and(|exact| {
                exact.source_identity == self.producer.recorded_source_identity
                    && exact.runtime_authority == self.producer.runtime_authority
                    && exact.schema == self.producer.schema
                    && exact.scope == self.producer.scope
                    && Arc::ptr_eq(&exact.correspondence, &self.correspondence)
            })
    }
}

/// Why a generated output did not qualify for suspension. Nothing changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputSuspensionDenial {
    /// The current output retains every entity, so there is no generated payload
    /// to suspend. The current output remains readable and no effect occurred.
    NoGeneratedPayload,
    /// The producer is not installed.
    ProducerUnavailable,
    /// No output from this producer is recorded at the current branch position.
    MissingQualifiedOutput,
    /// The source is not the current product source, or the recorded output
    /// came from a different source.
    SourceMismatch,
    /// The recorded output belongs to another runtime.
    ForeignRuntime,
    /// The recorded output belongs to another schema binding.
    ForeignSchema,
}

/// Why suspending a generated output did not complete.
pub enum WorthQueryGeneratedOutputSuspensionFailure {
    /// The host request was refused before observing generated output.
    ExecutionDenied(crate::domain_computation::request_execution::WorthQueryAdvancementDenial),
    /// The output did not qualify for suspension. Nothing changed.
    Qualification(WorthQueryGeneratedOutputSuspensionDenial),
    /// The product's program activation could not admit a publication. Nothing
    /// changed.
    ProductActivationUnavailable,
    /// The suspension could not be prepared. Carries the owning Relational
    /// denial; nothing was published and no materialization custody was issued.
    Preparation(worth_relational::facade::branch::RelationalMaterializationError),
    /// The publication was refused or had no effect. Nothing was published.
    PublicationNoEffect,
    /// Some owners moved, but the product head did not. Continue the recovery
    /// this carries.
    ProductUnpublished(WorthQueryGeneratedOutputSuspensionRecovery),
}

impl std::fmt::Debug for WorthQueryGeneratedOutputSuspensionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutionDenied(cause) => formatter
                .debug_tuple("ExecutionDenied")
                .field(cause)
                .finish(),
            Self::Qualification(denial) => formatter
                .debug_tuple("Qualification")
                .field(denial)
                .finish(),
            Self::ProductActivationUnavailable => {
                formatter.write_str("ProductActivationUnavailable")
            }
            Self::Preparation(cause) => formatter.debug_tuple("Preparation").field(cause).finish(),
            Self::PublicationNoEffect => formatter.write_str("PublicationNoEffect"),
            Self::ProductUnpublished(_) => formatter.write_str("ProductUnpublished"),
        }
    }
}

mod court_doors_for_test;
pub use reconstruction::ReconstructionOps;
pub use restoration::AdvancementOps;
pub use restoration::RestorationOps;
pub use suspension::RecoveryAdvancementOps;
