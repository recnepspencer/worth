use super::fixture_values;
use super::fixture_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarMutation {
    pub scope_key: String,
    pub operation: PlanarOperation,
}
worth_query_structured_value_binding!(pub PlanarMutationInputBinding for PlanarMutation { identity: "worth.query.certification.planar-mutation-input.v2" });
worth_query_operation!(pub MutatePlanar for Schema: TopologySchemaBinding, input PlanarMutationInputBinding);
worth_query_operation_reads!(MutatePlanar => [Body, BodyKey, PositionX, PositionY, Length, PlanarSuccessor]);
worth_query_operation_writes!(MutatePlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(MutatePlanar => [Body]);
worth_query_operation_links!(MutatePlanar => [PlanarSuccessor]);
worth_query_operation_unlinks!(MutatePlanar => [PlanarSuccessor]);

pub struct PlanarMutationBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarMutationBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = PlanarMutation;
    type InputBinding = PlanarMutationInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = MutatePlanar;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
    const IDENTITY: &'static str = "worth.query.certification.planar-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.planar-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.planar-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(16, 16, 16, 64, 8192);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PlanarMutation {
    type Binding = PlanarMutationBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
mod output_contract;
pub use output_contract::*;
