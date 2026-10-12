use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryNativeAccessLayout;
use crate::projection_consumption::{
    ProjectionConsumptionWarnings, WorthQueryConsumedProjectionAuthority,
};
use crate::runtime::WorthQueryWorkspace;

use super::{
    WorthQueryLiveProjectionRefreshError, WorthQueryLiveProjectionRefreshWork,
    WorthQueryProjectionLifecycleSource,
};

pub(super) struct PreparedRefreshProjection {
    pub(super) authority: Box<WorthQueryConsumedProjectionAuthority>,
    pub(super) source_rows: Vec<crate::memory_workspace::WorthQueryEntity>,
    pub(super) warnings: Option<ProjectionConsumptionWarnings>,
    pub(super) native_access: Option<WorthQueryNativeAccessLayout>,
    pub(super) work: WorthQueryLiveProjectionRefreshWork,
}

pub(super) fn prepare_projection<D: 'static, O: 'static, F: 'static, L: BasisOperationLane, S>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &S,
    handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
    workspace: &mut WorthQueryWorkspace,
    granular_read: Option<(
        &crate::live::WorthQueryMaintenanceScope,
        &crate::runtime::WorthQueryGranularSourceReadBasis,
    )>,
    mut work: WorthQueryLiveProjectionRefreshWork,
    owner_delivery_retained: bool,
) -> Result<PreparedRefreshProjection, WorthQueryLiveProjectionRefreshError>
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    work.begin_read();
    let read = match granular_read {
        Some((scope, basis)) => handle.read_granular_scope(execution, workspace, scope, basis),
        None => handle.read(execution, workspace),
    }
    .map_err(|error| WorthQueryLiveProjectionRefreshError::Runtime {
        error,
        work,
        owner_delivery_retained,
    })?;
    let source_rows = read.maintenance_source_rows().to_vec();
    let outcome = handle.project_contract(&read, source.projection_authority_contract());
    work.retain_projection();
    let (authority, warnings) = outcome.into_admitted().map_err(|outcome| {
        WorthQueryLiveProjectionRefreshError::Projection {
            outcome: Box::new(outcome),
            work,
            owner_delivery_retained,
        }
    })?;
    let native_layout = source.native_access_layout();
    if native_layout.is_some() {
        work.begin_native_rebind();
    }
    let native_access = native_layout
        .map(|layout| layout.rebind(source.consumer_contract(), &authority))
        .transpose()
        .map_err(
            |denial| WorthQueryLiveProjectionRefreshError::NativeAccess {
                denial,
                work,
                owner_delivery_retained,
            },
        )?;
    Ok(PreparedRefreshProjection {
        authority,
        source_rows,
        warnings,
        native_access,
        work,
    })
}
