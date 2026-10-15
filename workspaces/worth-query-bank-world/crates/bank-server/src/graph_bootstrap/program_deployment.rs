//! Production bootstrap issuance of exact, explicitly supplied deployment grants.
use super::*;
use crate::BankProgramDeploymentGrantSeed;
use worth_query_host::facade::declaration::application_program::ApplicationProgramRevision;
pub(crate) fn bind_program_deployment_scopes(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    revisions: &[ApplicationProgramRevision],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for revision in revisions {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                ProgramDeploymentScope::reference(),
                entity_key(scope_key(revision)),
            )
            .field(
                ProgramDeploymentSourceRevision::reference(),
                revision.to_string(),
            ),
        )?;
    }
    Ok(())
}
pub(crate) fn bind_program_deployment_grants(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    grants: &[BankProgramDeploymentGrantSeed],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for grant in grants {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                ProgramDeploymentGrant::reference(),
                entity_key(grant.key.as_str().to_owned()),
            )
            .field(
                ProgramDeploymentGrantIdentity::reference(),
                grant.key.as_str().to_owned(),
            )
            .field(
                ProgramDeploymentGrantSourceRevision::reference(),
                grant.source.to_string(),
            )
            .field(
                ProgramDeploymentGrantAction::reference(),
                "deploy-bank-program".to_owned(),
            )
            .field(
                ProgramDeploymentGrantPurpose::reference(),
                "program-deployment".to_owned(),
            )
            .field(
                ProgramDeploymentGrantStatus::reference(),
                "active".to_owned(),
            )
            .field(
                ProgramDeploymentGrantNotBefore::reference(),
                grant.not_before,
            )
            .field(ProgramDeploymentGrantNotAfter::reference(), grant.not_after)
            .field(ProgramDeploymentGrantDepth::reference(), 0),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantResource::reference(),
            format!("{}:resource", grant.key.as_str()),
            entity_key(grant.key.as_str().to_owned()),
            entity_key(scope_key(&grant.source)),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantGrantor::reference(),
            format!("{}:grantor", grant.key.as_str()),
            entity_key(principal_key(grant.grantor.get())),
            entity_key(grant.key.as_str().to_owned()),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ProgramDeploymentGrantGrantee::reference(),
            format!("{}:grantee", grant.key.as_str()),
            entity_key(principal_key(grant.grantee.get())),
            entity_key(grant.key.as_str().to_owned()),
        ))?;
    }
    Ok(())
}
fn scope_key(revision: &ApplicationProgramRevision) -> String {
    format!("bank-program-deployment:{revision}")
}
