//! Real authenticated principals and initial grants; later changes use native commands.
use super::{
    grants::*,
    handler::{Contacts, RetentionHandler},
    program::Program,
    schema::*,
};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_contribution::*,
    application_installation::*, declaration::application_program::*,
    declaration::authentication::*, primary_graph::*,
};
pub struct Host<P = Program> {
    pub application: WorthQueryProgramApplicationRuntime<CapabilitySchema, P>,
    pub contacts: Arc<Contacts>,
    pub projection: WorthQueryApplicationInvariantProjectionAuthority<CapabilitySchema>,
}
impl WorthQueryApplicationContribution<CapabilitySchema> for CapabilityContribution {
    type Configuration = Arc<Contacts>;
    fn configure(
        contacts: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CapabilitySchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SetRetentionBinding, _>(RetentionHandler(contacts.clone()))?;
        setup.handler::<super::workflow::bindings::GuardedBinding, _>(RetentionHandler(contacts))
    }
}
struct Adapter;
fn identity(actor: u64) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        "https://issuer.example",
        format!("capability-actor-{actor}"),
    )
    .unwrap()
}
impl WorthQueryAuthenticationAdapter for Adapter {
    type Credential = u64;
    fn configuration_identity(&self) -> &str {
        "capability-retention-auth"
    }
    fn validate<'a>(
        &'a self,
        actor: u64,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                identity(actor),
                WorthQueryAuthenticationAudience::new("capability-retention-host").unwrap(),
                WorthQueryAuthenticationMethod::new("certification").unwrap(),
                now,
                now + Duration::from_secs(3600),
                vec![],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}
pub fn authenticate<P>(
    host: &Host<P>,
    actor: u64,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema> {
    let adapter = admit_authentication_adapter(
        host.application.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("capability-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
        ),
        Adapter,
    )
    .unwrap();
    crate::document_retention_model::operator_identity::block_on(adapter.authenticate(actor, scope))
        .unwrap()
}
pub fn install() -> Host {
    install_program::<Program>(false)
}
pub fn install_program<
    P: ApplicationProgramDefinition<
        CapabilitySchema,
        Contributions = (CapabilityContribution,),
        Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>,
    >,
>(
    workflow: bool,
) -> Host<P> {
    let contacts = Arc::new(Contacts::default());
    let mut projection = None;
    let application = in_memory_rostered_program(
        ApplicationProgramAuthoring::<CapabilitySchema, P>::begin()
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        CapabilitySchema::declaration().unwrap(),
        (contacts.clone(),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        crate::document_retention_model::host::host_limits(),
        |_, graph, installed| {
            projection = Some(graph.retain_invariant_projection_authority());
            let binding = installed
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            for actor in [1, 2] {
                graph.bind_principal(
                    &binding,
                    WorthQueryApplicationPrincipalKey::new(format!("actor-{actor}")).unwrap(),
                    actor,
                    identity(actor),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
            }
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    WorthQueryApplicationEntityKey::new("document-row-1").unwrap(),
                )
                .field(
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                )
                .field(DocumentRetentionField::reference(), 7_u64),
            )?;
            seed_grant(graph, "source-child", 1, 1)?;
            seed_grant(graph, "issuer-revoke", 2, 2)?;
            if workflow {
                seed_grant(graph, "workflow-advancer", 1, 2)?;
            }
            Ok(())
        },
    )
    .unwrap();
    Host {
        application,
        contacts,
        projection: projection.expect("installation mints projection authority"),
    }
}
fn seed_grant(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CapabilitySchema>,
    name: &str,
    actor: u64,
    action: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Grant::reference(),
            WorthQueryApplicationEntityKey::new(name).unwrap(),
        )
        .field(GrantIdentity::reference(), name.to_owned())
        .field(GrantAction::reference(), action)
        .field(GrantPurpose::reference(), action)
        .field(GrantStatus::reference(), 1_u64)
        .field(GrantScopeKey::reference(), DOCUMENT_IDENTITY.to_owned())
        .field(GrantNotBefore::reference(), 0_u64)
        .field(GrantNotAfter::reference(), u64::MAX)
        .field(GrantDepth::reference(), 0_u64),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantResource::reference(),
        format!("{name}-resource"),
        WorthQueryApplicationEntityKey::new(name).unwrap(),
        WorthQueryApplicationEntityKey::new("document-row-1").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantee::reference(),
        format!("{name}-grantee"),
        WorthQueryApplicationEntityKey::new(format!("actor-{actor}")).unwrap(),
        WorthQueryApplicationEntityKey::new(name).unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantGrantor::reference(),
        format!("{name}-grantor"),
        WorthQueryApplicationEntityKey::new("actor-2").unwrap(),
        WorthQueryApplicationEntityKey::new(name).unwrap(),
    ))?;
    Ok(())
}
