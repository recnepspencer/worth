pub use worth_foundational::facade::ProducerPublicationOwnership;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationInputScope, ApplicationMutationScopeResolution,
    ApplicationQueryMutationSource, WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

pub mod demand;
pub mod synchronous_outputs;
pub use demand::VerifiedAppliedParent;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::denial_value;
pub use demand::MatchedRequiredPredecessors;
pub use demand::{CurrentBasis, CurrentProof};
pub use demand::{
    WorthQueryAdmittedOutputDemand, WorthQueryOriginalOutputDemand, WorthQueryOutputDemandAdvance,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture, WorthQueryOutputReadCleanup,
    WorthQueryOutputReadCloseDenial, WorthQueryOutputSettlementProvenanceDenial,
    WorthQuerySelectedApplicationProducer,
};
pub use denial_value::{
    WorthQueryProducerDenialDeclaration, WorthQueryProducerDenialDecodeDenial,
    WorthQueryProducerDenialEncodingRefusal, WorthQueryProducerDenialProjection,
    WorthQueryProducerDenialValue, WorthQueryProjectedProducerDenial,
    WorthQueryRecordedProducerDenialEncodingRefusal, WorthQueryRecordedProducerDenialValue,
    WorthQueryUndeclaredProducerDenial,
};
pub mod execution;
pub use execution::unpublished::PendingProducerPublication;
pub use execution::prepared_plan::OwnerPlan;
use execution::{InstalledProducerExecutor, TypedInstalledProducer};
pub use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::output_equivalence_contract;
pub mod output_evaluation;
pub use output_equivalence_contract::WorthQueryProducerOutputEquivalence;
pub use output_evaluation::ProducerOutputEvaluation;
pub use output_evaluation::WorthQueryProducerEvaluationIneligibility;
pub use output_evaluation::{ComparedProducerOutput, ProducerOutputEvaluationState};
pub mod observation_reads;
pub use observation_reads::{
    WorthQueryProducerObservationReadContract, WorthQueryProducerObservationReadDenial,
    WorthQueryProducerObservationReadOperation,
};
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::input_reuse_contract;
pub use input_reuse_contract::{
    WorthQueryDecisionContextDependencies, WorthQueryProducerInputReuseContract,
};

pub use worth_query_execution_lineage::facade::primary_graph::application_contribution::{
    WorthQueryProducerCommitAuthority, WorthQueryProducerInvariantRequirement,
};
mod readiness;
mod source_qualification_runtime;
pub use readiness::{
    evaluate_output_readiness, install_output_readiness_routes, PendingOutputReadiness,
    TypedPendingOutputReadiness, WorthQueryInstalledOutputReadinessRoutes,
};
pub mod scheduling;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::producer::source_qualification;
pub use scheduling::{
    schedule_output_producer, schedule_output_producer_on_selected,
    WorthQueryInstalledOutputProducerRoutes,
};
pub use source_qualification::QualifiedOutputSource;

pub use worth_query_declaration::facade::application_program::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerApplicability,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};

pub type ProducerSourceBinding<Schema, Binding> =
    <<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source;
pub type ProducerSourceQuery<Schema, Binding> = <<<Binding as WorthQueryApplicationProducerBinding<
    Schema,
>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<
    Schema,
>>::Query;
pub type ProducerSourceValue<Schema, Binding> = <<<<Binding as WorthQueryApplicationProducerBinding<
    Schema,
>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryWorkflowAssessmentPosture;

/// Assessment meaning owned by the installed output-family contract rather
/// than selected by a workflow caller.
pub trait WorthQueryWorkflowAssessmentOutputFamily<Schema>:
    WorthQueryProducerOutputFamily<Schema>
where
    Schema: ApplicationSchema,
{
    fn assessment_posture(
        source: &<<Self::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> WorthQueryWorkflowAssessmentPosture;
}

pub trait WorthQueryApplicationProducerProvider<Schema, Binding>: Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    const SEMANTIC_IDENTITY: &'static str;

    /// The declared canonical binding of the operation's domain denial, or
    /// `WorthQueryUndeclaredProducerDenial`. A rejected decision carries the
    /// encoded value to the demand's caller, who decodes it with this binding.
    type DenialValue: WorthQueryProducerDenialDeclaration<
        <Binding::Operation as ApplicationMutationBinding<Schema>>::Denial,
    >;

    fn operation_input(
        &self,
        source: &<<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> <Binding::Operation as ApplicationMutationBinding<Schema>>::Input;

    fn idempotency_key(
        &self,
        source: &<<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        source_identity: &[u8; 32],
    ) -> <Binding::Operation as ApplicationMutationBinding<Schema>>::IdempotencyKey;

    /// Optional domain-owned words for a rejected producer decision. This is
    /// diagnostic text beside the declared `DenialValue`, never an authority.
    /// Static text bounds retained diagnostic storage; no Debug text is parsed.
    fn domain_denial_reason(
        &self,
        _denial: &<Binding::Operation as ApplicationMutationBinding<Schema>>::Denial,
    ) -> Option<&'static str> {
        None
    }
}

pub trait WorthQueryApplicationProducerBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Operation: ApplicationMutationBinding<
        Schema,
        SourceExpectation = ApplicationQueryMutationSource<
            <<Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query,
        >,
        Input: ApplicationMutationInputScope<Schema, Self::Operation> + Clone + Send + Sync,
        ScopeBinding: ApplicationMutationScopeResolution<
            Schema,
            <Self::Operation as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
        Denial: std::fmt::Debug,
    >;
    type OutputFamily: WorthQueryProducerOutputFamily<Schema>;
    type Provider: WorthQueryApplicationProducerProvider<Schema, Self>;
    /// The fixed role of the operation's output contract that every commit
    /// binds to the family's entity. A role the contract does not declare,
    /// an at-most-one role, or a role of another entity fails to compile.
    type OutputRole: WorthQueryApplicationOutputRole<
        Schema = Schema,
        Contract = <Self::Operation as ApplicationMutationBinding<Schema>>::Output,
        Entity = <Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Entity,
        Cardinality = WorthQueryExactlyOneOutput,
    >;

    const IDENTITY: &'static str;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability];
    /// Invariants each publication must satisfy before it is admitted. A
    /// requirement is met when its rule passes, or when Relational proves the
    /// publication touches no kind in the rule's declared closure. A rule
    /// scoped out for any other reason, or missing, refuses the publication.
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement];
    const REUSE_POLICY: &'static str;
    /// SourceScopeCascade opts complete publications into authored cascade
    /// retirement. Installation requires an entity-specific source query scope.
    const OWNERSHIP: ProducerPublicationOwnership = ProducerPublicationOwnership::Independent;
    const PREDECESSOR_READS: Option<WorthQueryProducerObservationReadContract<Schema>> = None;
    /// Declared readonly observations of the admitted original C, not old consumed inputs.
    const ORIGINAL_READS: Option<WorthQueryProducerObservationReadContract<Schema>> = None;
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = None;

    /// The pure input preparation owned by this family. Configuration installs
    /// its original handler through `producer_with_input_capture`.
    fn input_capture() -> Option<WorthQueryProducerInputCaptureContract<Schema>> {
        None
    }

    /// Opt in to a bounded actual output-field evaluation. The field must belong
    /// to this producer's exactly-one output role; this grants no disclosure.
    fn output_equivalence() -> Option<WorthQueryProducerOutputEquivalence<Schema, Self::OutputRole>>
    {
        None
    }
}

pub mod input_capture;
pub use input_capture::InputCaptureMeaning;
pub use input_capture::{
    WorthQueryCapturedProducerInputs, WorthQueryProducerInputCaptureContract,
    WorthQueryProducerInputCaptureHandler, WorthQueryProducerInputReader,
};
pub mod registry;
pub use registry::DeclaredProducerBinding;
pub use registry::InstalledProducerEdition;
pub use registry::PendingProducerRegistry;
pub use registry::WorthQueryInstalledApplicationProducerRegistry;

pub use source_qualification::OutputSourceAddress;
#[cfg(test)]
#[path = "producer_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub use registry::RefusedProducerSeal;
