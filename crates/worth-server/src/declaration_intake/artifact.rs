use std::{fmt, sync::Mutex};
use worth_query::facade::foundation::{
    ProjectionAuthorityOutcome, ProjectionFactConsumptionPathError,
};
use worth_query::facade::runtime::{
    WorthQueryLiveReadResult, WorthQueryRuntimeError, WorthQueryRuntimePublicApiFamilyContract,
    WorthQueryRuntimeStateSnapshot, WorthQueryUnifiedInspectionResult, WorthQueryWorkspace,
};

use crate::{WorthServerAdmission, WorthServerResolvedRequestContext};

use super::{
    WorthServerDirectDeclaration, WorthServerDirectDeclarationDenial,
    WorthServerDirectDeclarationSourceSupportStatus, WorthServerDirectSupportSnapshot,
};
use crate::WorthServerDirectProjectionRequest;

pub(crate) enum WorthServerNamedLiveProjectionExecutionError {
    Runtime(Box<WorthQueryRuntimeError>),
    Consumption(Box<ProjectionFactConsumptionPathError>),
}

pub struct WorthServerPreparedDirectDeclaration {
    admission: WorthServerAdmission,
    declaration: WorthServerDirectDeclaration,
    workspace_name: String,
    workspace: Mutex<WorthQueryWorkspace>,
    declaration_digest: String,
    support_snapshot: WorthServerDirectSupportSnapshot,
}

impl fmt::Debug for WorthServerPreparedDirectDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthServerPreparedDirectDeclaration")
            .field("declaration", &self.declaration)
            .field("workspace_name", &self.workspace_name)
            .field("declaration_digest", &self.declaration_digest)
            .field("support_snapshot", &self.support_snapshot)
            .finish()
    }
}

impl WorthServerPreparedDirectDeclaration {
    pub(crate) fn new(
        admission: WorthServerAdmission,
        declaration: WorthServerDirectDeclaration,
        workspace: WorthQueryWorkspace,
        declaration_digest: String,
        support_snapshot: WorthServerDirectSupportSnapshot,
    ) -> Self {
        let workspace_name = workspace.name().to_string();
        Self {
            admission,
            declaration,
            workspace_name,
            workspace: Mutex::new(workspace),
            declaration_digest,
            support_snapshot,
        }
    }

    pub fn declaration(&self) -> &WorthServerDirectDeclaration {
        &self.declaration
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn support_snapshot(&self) -> &WorthServerDirectSupportSnapshot {
        &self.support_snapshot
    }

    pub fn resolved_request_context(&self) -> &WorthServerResolvedRequestContext {
        self.admission.resolved_request_context()
    }

    pub fn admission(&self) -> &WorthServerAdmission {
        &self.admission
    }

    pub fn admit(
        self,
    ) -> Result<WorthServerAdmittedDirectDeclaration, WorthServerDirectDeclarationDenial> {
        if self.support_snapshot.source_support_status()
            != WorthServerDirectDeclarationSourceSupportStatus::Supported
        {
            return Err(WorthServerDirectDeclarationDenial::source_not_admitted(
                self.admission.request_context().diagnostics_profile(),
                self.support_snapshot.source_support_reason(),
                self.support_snapshot.clone(),
            ));
        }

        if self.support_snapshot.read_family_contract().is_none()
            || !self.support_snapshot.read_family_pin_satisfied()
        {
            return Err(
                WorthServerDirectDeclarationDenial::query_facade_family_not_admitted(
                    self.admission.request_context().diagnostics_profile(),
                    "query workspace does not admit the read facade family for direct declaration intake",
                    self.support_snapshot.clone(),
                ),
            );
        }

        Ok(WorthServerAdmittedDirectDeclaration {
            admission: self.admission,
            declaration: self.declaration,
            workspace_name: self.workspace_name,
            workspace: self.workspace,
            declaration_digest: self.declaration_digest,
            support_snapshot: self.support_snapshot,
        })
    }
}

pub struct WorthServerAdmittedDirectDeclaration {
    admission: WorthServerAdmission,
    declaration: WorthServerDirectDeclaration,
    workspace_name: String,
    workspace: Mutex<WorthQueryWorkspace>,
    declaration_digest: String,
    support_snapshot: WorthServerDirectSupportSnapshot,
}

impl fmt::Debug for WorthServerAdmittedDirectDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthServerAdmittedDirectDeclaration")
            .field("declaration", &self.declaration)
            .field("workspace_name", &self.workspace_name)
            .field("declaration_digest", &self.declaration_digest)
            .field("support_snapshot", &self.support_snapshot)
            .finish()
    }
}

impl WorthServerAdmittedDirectDeclaration {
    pub fn declaration(&self) -> &WorthServerDirectDeclaration {
        &self.declaration
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn support_snapshot(&self) -> &WorthServerDirectSupportSnapshot {
        &self.support_snapshot
    }

    pub fn resolved_request_context(&self) -> &WorthServerResolvedRequestContext {
        self.admission.resolved_request_context()
    }

    pub fn query_family_contract(&self) -> &WorthQueryRuntimePublicApiFamilyContract {
        self.support_snapshot
            .read_family_contract()
            .expect("admitted direct declarations must retain an admitted read-family contract")
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub(crate) fn declaration_binding_label(&self) -> &str {
        self.declaration.source().binding_label()
    }

    pub(crate) fn execute_named_live_read(
        &self,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.with_workspace_mut(|workspace| {
            workspace.with_execution_request_mut(|execution, workspace| {
                let live_target =
                    workspace.resolve_live_artifact_target(self.declaration_binding_label())?;
                workspace.read_live_target(execution, &live_target)
            })
        })?
    }

    pub(crate) fn snapshot_named_live_state(
        &self,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        self.with_workspace(|workspace| {
            let live_target =
                workspace.resolve_live_artifact_target(self.declaration_binding_label())?;
            workspace.state_live_target(&live_target)
        })
    }

    pub(crate) fn inspect_named_live_view(
        &self,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        self.with_workspace(|workspace| {
            workspace.with_execution_request(|execution| {
                let live_target =
                    workspace.resolve_live_artifact_target(self.declaration_binding_label())?;
                workspace
                    .inspections()?
                    .inspect_live_target(execution, &live_target)
            })
        })?
    }

    pub(crate) fn subscription_basis_digest(&self) -> Result<String, WorthQueryRuntimeError> {
        self.with_workspace(|workspace| {
            let live_target =
                workspace.resolve_live_artifact_target(self.declaration_binding_label())?;
            workspace.subscription_basis_digest_for_target(&live_target)
        })
    }

    pub(crate) fn consume_named_live_projection(
        &self,
        request: &WorthServerDirectProjectionRequest,
    ) -> Result<ProjectionAuthorityOutcome, WorthServerNamedLiveProjectionExecutionError> {
        let live_read = self.with_workspace_mut(|workspace| {
            workspace
                .with_execution_request_mut(|execution, workspace| {
                    let live_target = workspace
                        .resolve_live_artifact_target(self.declaration_binding_label())
                        .map_err(|error| {
                            WorthServerNamedLiveProjectionExecutionError::Runtime(Box::new(error))
                        })?;
                    workspace
                        .read_live_target(execution, &live_target)
                        .map_err(|error| {
                            WorthServerNamedLiveProjectionExecutionError::Runtime(Box::new(error))
                        })
                })
                .map_err(|error| {
                    WorthServerNamedLiveProjectionExecutionError::Runtime(Box::new(error))
                })?
        })?;
        live_read
            .consume_projection_authority_with_binding(
                request.binding_context(
                    live_read.receipt().canonical_query_digest(),
                    live_read.receipt().view_shape_digest(),
                ),
                request.authority_contract_owned(),
            )
            .map_err(|error| {
                WorthServerNamedLiveProjectionExecutionError::Consumption(Box::new(error))
            })
    }

    fn with_workspace<T>(&self, operation: impl FnOnce(&WorthQueryWorkspace) -> T) -> T {
        let workspace = self
            .workspace
            .lock()
            .expect("direct declaration workspace mutex should not be poisoned");
        operation(&workspace)
    }

    fn with_workspace_mut<T>(&self, operation: impl FnOnce(&mut WorthQueryWorkspace) -> T) -> T {
        let mut workspace = self
            .workspace
            .lock()
            .expect("direct declaration workspace mutex should not be poisoned");
        operation(&mut workspace)
    }
}
