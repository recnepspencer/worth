//! Explicit native deployment grants for the fixed adoption consumer court.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramRevision,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
pub(crate) struct ConsumerDeploymentGrantSeed {
    source: ApplicationProgramRevision,
    grantor: String,
    grantee: String,
}
impl ConsumerDeploymentGrantSeed {
    pub(crate) fn for_program<Source: ApplicationProgramDefinition<ConsumerSchema>>(
        grantor: &str,
        grantee: &str,
    ) -> Self {
        Self {
            source: *ApplicationProgramAuthoring::<ConsumerSchema, Source>::begin()
                .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
                .validated_program()
                .expect("deployment grant names actual validated source meaning")
                .revision(),
            grantor: grantor.to_owned(),
            grantee: grantee.to_owned(),
        }
    }
    pub(crate) fn source(&self) -> ApplicationProgramRevision {
        self.source
    }
    pub(crate) fn bind(
        &self,
        graph: &mut WorthQueryPrimaryGraphBootstrap<ConsumerSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let scope = format!("consumer-deployment:{}", self.source);
        let grant = format!("{scope}:{}", self.grantee);
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(ProgramDeploymentGrant::reference(), key(&grant))
                .field(
                    ProgramDeploymentGrantIdentity::at("ProgramDeploymentGrantIdentity"),
                    grant.clone(),
                )
                .field(
                    ProgramDeploymentGrantSourceRevision::at(
                        "ProgramDeploymentGrantSourceRevision",
                    ),
                    self.source.to_string(),
                )
                .field(
                    ProgramDeploymentGrantAction::at("ProgramDeploymentGrantAction"),
                    "deploy-consumer-program".to_owned(),
                )
                .field(
                    ProgramDeploymentGrantPurpose::at("ProgramDeploymentGrantPurpose"),
                    "program-deployment".to_owned(),
                )
                .field(
                    ProgramDeploymentGrantStatus::at("ProgramDeploymentGrantStatus"),
                    "active".to_owned(),
                )
                .field(
                    ProgramDeploymentGrantNotBefore::at("ProgramDeploymentGrantNotBefore"),
                    0,
                )
                .field(
                    ProgramDeploymentGrantNotAfter::at("ProgramDeploymentGrantNotAfter"),
                    u64::MAX,
                )
                .field(
                    ProgramDeploymentGrantDepth::at("ProgramDeploymentGrantDepth"),
                    0,
                ),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantResource::reference(),
            format!("{grant}:resource"),
            key(&grant),
            key(&scope),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantGrantor::reference(),
            format!("{grant}:grantor"),
            key(&self.grantor),
            key(&grant),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantGrantee::reference(),
            format!("{grant}:grantee"),
            key(&self.grantee),
            key(&grant),
        ))?;
        Ok(())
    }
}
fn key<Entity>(identity: &str) -> WorthQueryApplicationEntityKey<ConsumerSchema, Entity> {
    WorthQueryApplicationEntityKey::new(identity.to_owned())
        .expect("fixture deployment entity identity is valid")
}

pub(crate) fn bind_scope(
    graph: &mut WorthQueryPrimaryGraphBootstrap<ConsumerSchema>,
    source: ApplicationProgramRevision,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let scope = format!("consumer-deployment:{source}");
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(ProgramDeploymentScope::reference(), key(&scope))
            .field(
                ProgramDeploymentSourceRevision::at("ProgramDeploymentSourceRevision"),
                source.to_string(),
            ),
    )
}
