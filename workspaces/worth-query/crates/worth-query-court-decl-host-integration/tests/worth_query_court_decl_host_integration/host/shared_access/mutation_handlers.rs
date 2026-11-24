//! Operation handlers and application program declaration for the shared-access court.

use worth_query_decl::facade::{
    application_program::*,
    authentication::{WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus},
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryApplicationEntityKey, WorthQueryInvariantMutationTarget,
        WorthQueryOperationDurableCapability, WorthQueryPrimaryGraphInstallationDenial,
    },
};

use super::mutations::{
    GrantProjectAccessBinding, GrantProjectDecision, GrantProjectInput, RevokeProjectAccessBinding,
    RevokeProjectInput, WriteProjectInput, WriteProjectVersionBinding, OWNER_PRINCIPAL_ID,
};
use super::query::{ProjectUpdatedEffect, ProjectUpdatedNotice};
use super::schema::{
    ExternalIdentityField, GrantActionField, GrantActiveField, GrantDelegationLimitField,
    GrantDisclosureField, GrantIdField, GrantModeField, GrantNotAfterField, GrantNotBeforeField,
    GrantPurposeField, GrantResource, GrantWorkflowField, Grantee, Grantor, MappingStatusField,
    MappingTarget, PrincipalIdField, Project, ProjectGrant, ProjectKeyField, ProjectVersionField,
    SharedProjectSchema, SharedProjectSurface,
};

pub struct WriteProjectHandler;
impl OperationHandler<SharedProjectSchema, WriteProjectVersionBinding> for WriteProjectHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &WriteProjectInput,
        reader: &mut DecisionReader<'_, '_, '_, SharedProjectSchema, WriteProjectVersionBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>, ()> {
        let project = match reader
            .resolve_entity(ProjectKeyField::at("ProjectKey"), input.project_key.clone())
        {
            Ok(p) => p,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&project, ProjectVersionField::at("ProjectVersion")) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&project) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &WriteProjectInput,
        target: WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>,
        writer: &mut CandidateWriter<'_, SharedProjectSchema, WriteProjectVersionBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(e) => e,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if let Err(cause) = writer.write_field(
            &entity,
            ProjectVersionField::at("ProjectVersion"),
            input.next_version,
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.emit(
            ProjectUpdatedEffect::reference(),
            ProjectUpdatedNotice {
                project_key: input.project_key.clone(),
            },
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        HandlerResult::Completed(input.next_version)
    }
}

pub struct GrantProjectHandler;
impl OperationHandler<SharedProjectSchema, GrantProjectAccessBinding> for GrantProjectHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &GrantProjectInput,
        reader: &mut DecisionReader<'_, '_, '_, SharedProjectSchema, GrantProjectAccessBinding>,
    ) -> HandlerResult<GrantProjectDecision, ()> {
        let owner = match reader
            .resolve_entity(PrincipalIdField::at("PrincipalIdField"), OWNER_PRINCIPAL_ID)
        {
            Ok(p) => p,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let grantee = match resolve_enabled_grantee(reader, input) {
            Ok(p) => p,
            Err(res) => return res,
        };
        let project = match reader
            .resolve_entity(ProjectKeyField::at("ProjectKey"), input.project_key.clone())
        {
            Ok(p) => p,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let (owner, grantee, project) = match (
            reader.mutation_target(&owner),
            reader.mutation_target(&grantee),
            reader.mutation_target(&project),
        ) {
            (Ok(o), Ok(g), Ok(p)) => (o, g, p),
            (Err(c), _, _) | (_, Err(c), _) | (_, _, Err(c)) => {
                return HandlerResult::ExecutionDenied(c)
            }
        };
        HandlerResult::Completed(GrantProjectDecision {
            owner,
            grantee,
            project,
        })
    }
    fn build_candidate(
        &self,
        input: &GrantProjectInput,
        decision: GrantProjectDecision,
        writer: &mut CandidateWriter<'_, SharedProjectSchema, GrantProjectAccessBinding>,
    ) -> HandlerResult<u64, ()> {
        let owner = match writer.projected_entity(&decision.owner) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        let grantee = match writer.projected_entity(&decision.grantee) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        let project = match writer.projected_entity(&decision.project) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        let key = WorthQueryApplicationEntityKey::new(input.grant_id.clone()).unwrap();
        let grant = match writer.create_entity(ProjectGrant::reference(), key) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        let res = init_grant_fields(writer, &grant, input).and_then(|()| {
            writer
                .link(
                    Grantor::reference(),
                    format!("grantor-{}", input.grant_id),
                    &owner,
                    &grant,
                )
                .and_then(|()| {
                    writer.link(
                        Grantee::reference(),
                        format!("grantee-{}", input.grant_id),
                        &grantee,
                        &grant,
                    )
                })
                .and_then(|()| {
                    writer.link(
                        GrantResource::reference(),
                        format!("resource-{}", input.grant_id),
                        &grant,
                        &project,
                    )
                })
        });
        match res {
            Ok(()) => HandlerResult::Completed(1),
            Err(c) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        }
    }
}

fn resolve_enabled_grantee(
    reader: &mut DecisionReader<'_, '_, '_, SharedProjectSchema, GrantProjectAccessBinding>,
    input: &GrantProjectInput,
) -> Result<
    worth_query_host::facade::primary_graph::WorthQueryInvariantEntityIdentity<
        SharedProjectSchema,
        super::schema::Principal,
    >,
    HandlerResult<GrantProjectDecision, ()>,
> {
    let ext = WorthQueryExternalPrincipalIdentity::new(
        input.grantee_issuer.clone(),
        input.grantee_subject.clone(),
    )
    .map_err(|_| HandlerResult::DomainDenied(()))?;
    let mapping = reader
        .resolve_entity(ExternalIdentityField::at("ExternalIdentityField"), ext)
        .map_err(|_| HandlerResult::DomainDenied(()))?;
    let status = reader
        .field(&mapping, MappingStatusField::at("MappingStatusField"))
        .map_err(HandlerResult::ExecutionDenied)?;
    if status != Some(WorthQueryPrincipalMappingStatus::Enabled) {
        return Err(HandlerResult::DomainDenied(()));
    }
    let mut targets = reader
        .relations_from(MappingTarget::reference(), &mapping)
        .map_err(|_| HandlerResult::DomainDenied(()))?;
    if targets.len() != 1 {
        return Err(HandlerResult::DomainDenied(()));
    }
    Ok(targets.pop().unwrap().into_to())
}

fn init_grant_fields(
    writer: &mut CandidateWriter<'_, SharedProjectSchema, GrantProjectAccessBinding>,
    grant: &worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity<
        SharedProjectSchema,
        ProjectGrant,
    >,
    input: &GrantProjectInput,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenial> {
    writer.initialize_field(grant, GrantIdField::at("GrantId"), input.grant_id.clone())?;
    writer.initialize_field(
        grant,
        GrantActionField::at("GrantAction"),
        "access".to_owned(),
    )?;
    writer.initialize_field(
        grant,
        GrantPurposeField::at("GrantPurpose"),
        "shared".to_owned(),
    )?;
    writer.initialize_field(
        grant,
        GrantDisclosureField::at("GrantDisclosure"),
        "project_version".to_owned(),
    )?;
    writer.initialize_field(grant, GrantModeField::at("GrantMode"), input.mode.clone())?;
    writer.initialize_field(grant, GrantActiveField::at("GrantActive"), true)?;
    writer.initialize_field(
        grant,
        GrantWorkflowField::at("GrantWorkflow"),
        input.project_key.clone(),
    )?;
    writer.initialize_field(grant, GrantNotBeforeField::at("GrantNotBefore"), 0_u64)?;
    writer.initialize_field(grant, GrantNotAfterField::at("GrantNotAfter"), u64::MAX)?;
    writer.initialize_field(
        grant,
        GrantDelegationLimitField::at("GrantDelegationLimit"),
        0_u64,
    )
}

pub struct RevokeProjectHandler;
impl OperationHandler<SharedProjectSchema, RevokeProjectAccessBinding> for RevokeProjectHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &RevokeProjectInput,
        reader: &mut DecisionReader<'_, '_, '_, SharedProjectSchema, RevokeProjectAccessBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<SharedProjectSchema, ProjectGrant>, ()>
    {
        let grant = match reader.resolve_entity(GrantIdField::at("GrantId"), input.grant_id.clone())
        {
            Ok(g) => g,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if let Err(cause) = reader.field(&grant, GrantActiveField::at("GrantActive")) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&grant) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &RevokeProjectInput,
        target: WorthQueryInvariantMutationTarget<SharedProjectSchema, ProjectGrant>,
        writer: &mut CandidateWriter<'_, SharedProjectSchema, RevokeProjectAccessBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(e) => e,
            Err(c) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        };
        match writer.write_field(&entity, GrantActiveField::at("GrantActive"), false) {
            Ok(()) => HandlerResult::Completed(1),
            Err(c) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        }
    }
}

impl WorthQueryApplicationContribution<SharedProjectSchema> for SharedProjectSurface {
    type Configuration = ();
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, SharedProjectSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<WriteProjectVersionBinding, _>(WriteProjectHandler)?;
        setup.handler::<GrantProjectAccessBinding, _>(GrantProjectHandler)?;
        setup.handler::<RevokeProjectAccessBinding, _>(RevokeProjectHandler)?;
        Ok(())
    }
}

pub struct SharedProjectProgram;
pub struct SharedFeature;
pub struct WriteCompletion;
pub struct GrantCompletion;
pub struct RevokeCompletion;

impl ApplicationActionDelivery<SharedProjectSchema, WriteProjectVersionBinding>
    for WriteCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.court.shared_access.write_completion.v1";
}
impl ApplicationActionDelivery<SharedProjectSchema, GrantProjectAccessBinding> for GrantCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.court.shared_access.grant_completion.v1";
}
impl ApplicationActionDelivery<SharedProjectSchema, RevokeProjectAccessBinding>
    for RevokeCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.court.shared_access.revoke_completion.v1";
}
impl ApplicationFeature<SharedProjectSchema> for SharedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.court.shared_access.feature.v1";
}
impl ApplicationProgramDefinition<SharedProjectSchema> for SharedProjectProgram {
    type Contributions = (SharedProjectSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.shared_access.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<SharedProjectSchema, SharedFeature>()
                .action::<WriteProjectVersionBinding, WriteCompletion>()
                .finish_action()
                .action::<GrantProjectAccessBinding, GrantCompletion>()
                .finish_action()
                .action::<RevokeProjectAccessBinding, RevokeCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}
