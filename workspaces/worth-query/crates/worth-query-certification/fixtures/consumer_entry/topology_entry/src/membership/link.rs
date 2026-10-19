use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_links, worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::super::*;
use super::MembershipSchemaBinding;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarDiscoveryLink {
    pub scope_key: String,
    pub target_key: String,
}

worth_query_structured_value_binding!(pub PlanarDiscoveryLinkInputBinding for PlanarDiscoveryLink {
    identity: "worth.query.certification.planar-discovery-link-input.v1"
});
worth_query_structured_value_binding!(pub PlanarDiscoveryLinkResultBinding for PlanarAdjustmentResult {
    identity: "worth.query.certification.planar-discovery-link-result.v1"
});
worth_query_structured_value_binding!(pub PlanarDiscoveryLinkDenialBinding for PlanarMutationDenial {
    identity: "worth.query.certification.planar-discovery-link-denial.v1"
});
worth_query_operation!(pub LinkPlanarDiscovery for Schema: MembershipSchemaBinding, input PlanarDiscoveryLinkInputBinding);
worth_query_operation_reads!(LinkPlanarDiscovery => [Body, BodyKey, PlanarDiscoverySource]);
worth_query_operation_links!(LinkPlanarDiscovery => [PlanarDiscoverySource]);

pub struct PlanarDiscoveryLinkBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: MembershipSchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarDiscoveryLinkBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = PlanarDiscoveryLink;
    type InputBinding = PlanarDiscoveryLinkInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarDiscoveryLinkResultBinding;
    type IdempotencyKey = u64;
    type Operation = LinkPlanarDiscovery;
    type Decision = [WorthQueryInvariantMutationTarget<Schema, Body>; 2];
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarDiscoveryLinkDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.planar-discovery-link.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.planar-discovery-link-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.planar-discovery-link-command.v1";
    const CANDIDATES:
        worth_query_decl::facade::application_operation::ApplicationCandidateContract =
        worth_query_decl::facade::application_operation::ApplicationCandidateContract::FixedShape(
            requirements(0, 1, 0, 0, 1024),
        );

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

impl<Schema: MembershipSchemaBinding> ApplicationMutationIntent<Schema> for PlanarDiscoveryLink {
    type Binding = PlanarDiscoveryLinkBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub struct PlanarDiscoveryLinkHandler;

impl<Schema: MembershipSchemaBinding> OperationHandler<Schema, PlanarDiscoveryLinkBinding<Schema>>
    for PlanarDiscoveryLinkHandler
{
    fn decide(
        &self,
        input: &PlanarDiscoveryLink,
        reader: &mut DecisionReader<'_, '_, '_, Schema, PlanarDiscoveryLinkBinding<Schema>>,
    ) -> HandlerResult<[WorthQueryInvariantMutationTarget<Schema, Body>; 2], PlanarMutationDenial>
    {
        let result = (|| {
            let from = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let to = reader.resolve_entity(BodyKey::reference(), input.target_key.clone())?;
            reader
                .relations_from(PlanarDiscoverySource::reference(), &from)
                .map_err(HandlerExecutionDenial::new)?;
            Ok([reader.mutation_target(&from)?, reader.mutation_target(&to)?])
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn candidate_requirements(
        &self,
        _: &PlanarDiscoveryLink,
        _: &[WorthQueryInvariantMutationTarget<Schema, Body>; 2],
    ) -> ApplicationCandidateRequirements {
        requirements(0, 1, 0, 0, 1024)
    }
    fn build_candidate(
        &self,
        input: &PlanarDiscoveryLink,
        targets: [WorthQueryInvariantMutationTarget<Schema, Body>; 2],
        writer: &mut CandidateWriter<'_, Schema, PlanarDiscoveryLinkBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let from = writer
                .projected_entity(&targets[0])
                .map_err(HandlerExecutionDenial::new)?;
            let to = writer
                .projected_entity(&targets[1])
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    PlanarDiscoverySource::reference(),
                    format!("membership:{}:{}", input.scope_key, input.target_key),
                    &from,
                    &to,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(PlanarAdjustmentResult {
                changed_vertices: 0,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

pub(crate) fn declare_planar_discovery_link<Schema: MembershipSchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = LinkPlanarDiscovery::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 128)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_relation(operation, PlanarDiscoverySource::reference())
        .operation_link(operation, PlanarDiscoverySource::reference())
        .application_mutation_binding::<PlanarDiscoveryLinkBinding<Schema>>()
}

impl<Schema: MembershipSchemaBinding>
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>
    for PlanarDiscoveryLinkBinding<Schema>
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationConventionalAdmission;
}
