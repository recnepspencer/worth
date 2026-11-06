pub use worth_foundational::facade::ProducerPublicationOwnership;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationInputScope, ApplicationMutationScopeResolution,
    ApplicationQueryMutationSource, WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

pub(crate) mod demand;
mod denial_value;
pub(in crate::domain_computation::primary_graph) use demand::CurrentProof;
pub(in crate::domain_computation::primary_graph) use demand::MatchedRequiredPredecessors;
pub use {
    demand::WorthQueryAdmittedOutputDemand, demand::WorthQueryOutputDemandAdvance,
    demand::WorthQuerySelectedApplicationProducer,
};
pub use {
    denial_value::WorthQueryProducerDenialDeclaration,
    denial_value::WorthQueryProducerDenialProjection,
    denial_value::WorthQueryProjectedProducerDenial,
    denial_value::WorthQueryUndeclaredProducerDenial,
};
pub(crate) mod execution;
pub(in crate::domain_computation::primary_graph) use execution::unpublished::PendingProducerPublication;
pub(crate) mod invocation;
pub(in crate::domain_computation::primary_graph) use execution::prepared_plan::OwnerPlan;
use execution::{InstalledProducerExecutor, TypedInstalledProducer};
pub(in crate::domain_computation::primary_graph) use invocation::ProducerInvocationAuthority;
pub(crate) mod output_equivalence_contract;
pub(crate) mod output_evaluation;
pub use output_equivalence_contract::WorthQueryProducerOutputEquivalence;
pub use output_evaluation::WorthQueryProducerEvaluationIneligibility;
pub(in crate::domain_computation::primary_graph) use output_evaluation::{
    ComparedProducerOutput, ProducerOutputEvaluation, ProducerOutputEvaluationState,
};
mod input_reuse_contract;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract;

mod readiness;
pub(in crate::domain_computation::primary_graph) use readiness::{
    evaluate_output_readiness, install_output_readiness_routes, PendingOutputReadiness,
    TypedPendingOutputReadiness, WorthQueryInstalledOutputReadinessRoutes,
};
pub(crate) mod scheduling;
mod source_qualification;
pub(in crate::domain_computation::primary_graph) use scheduling::{
    schedule_output_producer, schedule_output_producer_on_selected,
    WorthQueryInstalledOutputProducerRoutes,
};
pub(in crate::domain_computation::primary_graph) use source_qualification::QualifiedOutputSource;

pub trait WorthQueryProducerOutputFamily<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationQueryBinding<Schema>;
    /// The entity every producer of this family outputs. A producer whose
    /// output role names another entity fails to compile.
    type Entity: ApplicationEntityMarkerIdentity<Schema> + 'static;

    const IDENTITY: &'static str;
    const SUPPORTED: &'static [WorthQueryProducerApplicability];

    fn profile_kind(
        source: &<<Self::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> &'static str;
}

pub(super) type ProducerSourceBinding<Schema, Binding> =
    <<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source;
pub(super) type ProducerSourceQuery<Schema, Binding> =
    <<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query;
pub(super) type ProducerSourceValue<Schema, Binding> =
    <<<<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

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

/// Typed request for one installed output family from one authored source.
pub trait WorthQueryApplicationOutputDemand<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type OutputFamily: WorthQueryProducerOutputFamily<Schema>;

    fn source_intent(
        &self,
    ) -> <<Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Input;
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
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = None;

    /// Opt in to a bounded actual output-field evaluation. The field must belong
    /// to this producer's exactly-one output role; this grants no disclosure.
    fn output_equivalence() -> Option<WorthQueryProducerOutputEquivalence<Schema, Self::OutputRole>>
    {
        None
    }
}

pub(crate) mod registry;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerApplicability;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryWorkflowAssessmentPosture;
pub use demand::AdmissionOps;
pub(in crate::domain_computation) use demand::AdmissionScopedOps;
pub(in crate::domain_computation) use demand::DependentScopedOps;
pub use demand::ProgressionOps;
pub(in crate::domain_computation::primary_graph) use demand::ProgressionScopedOps;
pub use demand::RetainedObservationOps;
pub use demand::SelectedProgramOps;
pub use demand::SelectionOps;
pub use demand::SourceCustodyOps;
pub(in crate::domain_computation::primary_graph) use demand::SourceCustodyScopedOps;
pub(in crate::domain_computation::primary_graph) use demand::SourceRecoveryScopedOps;
pub(in crate::domain_computation::primary_graph) use demand::UnavailableScopedOps;
pub(super) use registry::DeclaredProducerBinding;

pub(in crate::domain_computation::primary_graph) use registry::PendingProducerRegistry;
pub use registry::WorthQueryInstalledApplicationProducerRegistry;
pub(in crate::domain_computation::primary_graph) use source_qualification::SourceQualificationScopedOps;
