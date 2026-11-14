//! Static schema meaning, principal mapping and native body ownership.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
};
pub struct Schema;
impl fragment::Fragment for Schema {
    const NAME: &'static str = "workflow_output";
    type Bindings = fragment::NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "workflow_output_court";
    const NAME: &'static str = "workflow_output";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = (Contribution,);
}
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("durable-workflow.output-model.v1");
    type Members = (
        RetentionModel,
        WorkflowGrants,
        document::document_history::control_roles::ControlGrants,
        Declaration,
    );
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.operation_body::<Binding>()
    }
}
pub struct Declaration;
impl fragment::Registration<Schema> for Declaration {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let body = OutputOperation::reference()
            .body()
            .in_scope(record())
            .unwrap()
            .preserve_scope_output::<Binding, DocumentOutput>()
            .unwrap()
            .finish()
            .unwrap();
        Ok(schema
            .principal_binding(authentication())
            .operation(
                OutputOperation::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .unwrap(),
            )
            .operation_projection_work_budget(OutputOperation::reference(), 8)
            .operation_read_field(
                OutputOperation::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .application_mutation_binding::<Binding>()
            .operation(
                ControlOperation::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(ControlOperation::reference(), 512)
            .operation_read_field(
                ControlOperation::reference(),
                DocumentIdentityField::at("DocumentIdentityField"),
            )
            .application_mutation_binding::<ControlBinding>()
            .application_mutation_binding::<ControlBinding<StartCapability>>()
            .application_mutation_binding::<ControlBinding<AdvanceCapability>>()
            .capability_context(ControlContext::reference())
            .capability_provenance(ControlProvenance::reference())
            .capability(contract::<ControlCapability>())
            .capability(contract::<StartCapability>())
            .capability(contract::<AdvanceCapability>()))
    }
}
pub struct Authentication;
pub fn authentication() -> ApplicationPrincipalBindingRef<
    Schema,
    Authentication,
    ExternalMapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "output-principal",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalIdentityField::at("ExternalIdentityField"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                MappingStatusField::at("MappingStatusField"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(fragment::relation(
                RetentionModel,
                ExternalMapping,
                MappingTarget,
            )),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                PrincipalIdentityField::at("PrincipalIdentityField"),
            ),
        },
    )
}
pub fn record() -> ApplicationEntityRef<Schema, Document> {
    fragment::entity(RetentionModel, Document)
}
pub fn principal() -> ApplicationEntityRef<Schema, Principal> {
    fragment::entity(RetentionModel, Principal)
}
pub fn grant() -> ApplicationEntityRef<Schema, WorkflowAuthoringGrant> {
    fragment::entity(WorkflowGrants, WorkflowAuthoringGrant)
}
pub fn owner() -> ApplicationRelationRef<Schema, WorkflowDocumentOwner, Principal, Document> {
    fragment::relation(RetentionModel, Principal, WorkflowDocumentOwner)
}
pub fn resource(
) -> ApplicationRelationRef<Schema, WorkflowGrantResource, WorkflowAuthoringGrant, Document> {
    fragment::relation(
        WorkflowGrants,
        WorkflowAuthoringGrant,
        WorkflowGrantResource,
    )
}
pub fn related(
) -> ApplicationRelationRef<Schema, WorkflowGrantRelated, WorkflowAuthoringGrant, Document> {
    fragment::relation(WorkflowGrants, WorkflowAuthoringGrant, WorkflowGrantRelated)
}
pub fn parent() -> ApplicationRelationRef<
    Schema,
    WorkflowGrantParent,
    WorkflowAuthoringGrant,
    WorkflowAuthoringGrant,
> {
    fragment::relation(WorkflowGrants, WorkflowAuthoringGrant, WorkflowGrantParent)
}
pub fn grantor(
) -> ApplicationRelationRef<Schema, WorkflowGrantGrantor, Principal, WorkflowAuthoringGrant> {
    fragment::relation(RetentionModel, Principal, WorkflowGrantGrantor)
}
pub fn grantee(
) -> ApplicationRelationRef<Schema, WorkflowGrantGrantee, Principal, WorkflowAuthoringGrant> {
    fragment::relation(RetentionModel, Principal, WorkflowGrantGrantee)
}
pub struct Program;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "durable-workflow.output-feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("durable-workflow.output-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<ControlBinding>()
            .finish_action()
            .native_action::<ControlBinding<StartCapability>>()
            .finish_action()
            .native_action::<ControlBinding<AdvanceCapability>>()
            .finish_action()
            .native_action::<Binding>()
            .finish_action()
            .finish()]
    }
}
pub struct Spec;
impl ApplicationWorkflowSpec for Spec {
    type Schema = Schema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("durable-workflow.output-spec.v1");
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::governed(
            ApplicationWorkflowReadRule::subject::<Self, Read>(),
            ApplicationWorkflowReadRule::definition::<Self, Read>(),
            None,
            ApplicationWorkflowAuditPolicy::NotApplicable,
        )
    }
}
pub struct Read;
impl ApplicationWorkflowReadBinding<Schema> for Read {
    const IDENTITY: &'static str = "durable-workflow.output-read.v1";
    type Capability = ControlCapability;
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Operation = ControlOperation;
    type PrincipalBinding = Authentication;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Authentication,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        authentication()
    }
}
