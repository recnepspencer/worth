use crate::runtime::WorthQueryWorkspace;

use super::WorthQueryWorkflowEffectEvidence;

/// The workflow-stage workspace surface. Its only executable doors live on
/// the stage context and therefore enforce declared reads and effects first.
pub struct WorthQueryWorkflowStageWorkspace<'a> {
    pub(super) workspace: &'a mut WorthQueryWorkspace,
    pub(super) artifact_production_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactProductionAuthority>>,
    pub(super) artifact_access_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactAccessAuthority>>,
    // The owner retains request refusal even if authored apply renders it as text.
    pub(super) request_denial: Option<Box<super::WorthQueryWorkflowAdvanceDenialKind>>,
    pub(super) installed_read_executions: usize,
    pub(super) executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
}

impl<'a> WorthQueryWorkflowStageWorkspace<'a> {
    pub(crate) fn new(
        workspace: &'a mut WorthQueryWorkspace,
        artifact_production_authority: Option<
            std::sync::Arc<crate::domain_installation::WorthQueryArtifactProductionAuthority>,
        >,
        artifact_access_authority: Option<
            std::sync::Arc<crate::domain_installation::WorthQueryArtifactAccessAuthority>,
        >,
    ) -> Self {
        Self {
            workspace,
            artifact_production_authority,
            artifact_access_authority,
            request_denial: None,
            installed_read_executions: 0,
            executed_effects: Vec::new(),
        }
    }

    pub(crate) fn installed_read_executions(&self) -> usize {
        self.installed_read_executions
    }

    pub fn artifact_reader<'b>(
        &'b self,
        artifact: &'b crate::domain_installation::WorthQueryTransferredArtifactHandle,
    ) -> Result<
        crate::domain_installation::WorthQueryStageArtifactReader<'b>,
        crate::domain_installation::WorthQueryArtifactNativeAccessDenial,
    > {
        let Some(authority) = self.artifact_access_authority.as_deref() else {
            return Err(
                crate::domain_installation::WorthQueryArtifactNativeAccessDenial::new(
                    crate::domain_installation::WorthQueryArtifactNativeAccessDenialKind::AccessPathDenied,
                    None,
                    "workflow stage has no installed artifact input access authority",
                    crate::domain_installation::WorthQueryArtifactNativeAccessCounters::default(),
                ),
            );
        };
        crate::domain_installation::WorthQueryStageArtifactReader::admit(artifact, authority)
    }

    pub fn register_artifact<R: crate::domain_installation::WorthQueryArtifactProviderResource>(
        &mut self,
        admission: crate::domain_installation::WorthQueryArtifactProductionAdmission,
        resource: R,
    ) -> Result<
        crate::domain_installation::WorthQueryMoveOnlyArtifactHandle,
        crate::domain_installation::WorthQueryArtifactDenial,
    > {
        self.register_artifact_resource(admission, resource)
    }

    pub(super) fn register_artifact_resource<
        R: crate::domain_installation::WorthQueryArtifactProviderResource,
    >(
        &self,
        admission: crate::domain_installation::WorthQueryArtifactProductionAdmission,
        resource: R,
    ) -> Result<
        crate::domain_installation::WorthQueryMoveOnlyArtifactHandle,
        crate::domain_installation::WorthQueryArtifactDenial,
    > {
        let expected = self
            .artifact_production_authority
            .as_ref()
            .ok_or_else(|| {
                crate::domain_installation::WorthQueryArtifactDenial::new(
                    crate::domain_installation::WorthQueryArtifactDenialKind::ArtifactContractNotInstalled,
                    None,
                    "workflow stage has no artifact production authority",
                )
            })?;
        crate::domain_installation::WorthQueryArtifactProductionAuthority::register_exact(
            expected, admission, resource,
        )
    }

    pub(crate) fn into_executed_effects(self) -> Vec<WorthQueryWorkflowEffectEvidence> {
        self.executed_effects
    }
}
