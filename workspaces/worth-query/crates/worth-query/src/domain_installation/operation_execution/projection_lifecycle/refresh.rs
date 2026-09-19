use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryNativeAccessDenial, WorthQueryNativeAccessKey, WorthQueryNativeAccessLayout,
    WorthQueryNativeFieldAccess,
};
use crate::ordinary::read::WorthQueryProjectionOutcome;
use crate::projection_consumption::{
    ProjectionConsumptionWarnings, WorthQueryConsumedProjectionAuthority,
};
use crate::runtime::{WorthQueryRuntimeError, WorthQueryWorkspace};

use super::refresh_work::WorthQueryLiveProjectionRefreshWork;
use super::source::WorthQueryProjectionLifecycleSource;
use super::{
    WorthQueryLiveBoundDomainProjection, WorthQueryLiveBoundWorkflowProjection,
    WorthQueryProjectionPromotionDenialKind,
};

#[path = "refresh/projection.rs"]
mod projection;

pub struct WorthQueryLiveProjectionRefresh {
    authority: Box<WorthQueryConsumedProjectionAuthority>,
    source_rows: Vec<crate::memory_workspace::WorthQueryEntity>,
    warnings: Option<ProjectionConsumptionWarnings>,
    native_access: Option<WorthQueryNativeAccessLayout>,
    delivery: crate::ordinary::live::WorthQueryManagedLiveDelivery,
    work: WorthQueryLiveProjectionRefreshWork,
    impact: std::sync::Arc<crate::domain_installation::WorthQueryImpactDecision>,
}

impl WorthQueryLiveProjectionRefresh {
    pub fn authority(&self) -> &WorthQueryConsumedProjectionAuthority {
        &self.authority
    }

    pub(crate) fn source_rows(&self) -> &[crate::memory_workspace::WorthQueryEntity] {
        &self.source_rows
    }

    pub fn warnings(&self) -> Option<&ProjectionConsumptionWarnings> {
        self.warnings.as_ref()
    }

    pub fn delivery(&self) -> &crate::ordinary::live::WorthQueryManagedLiveDelivery {
        &self.delivery
    }

    pub fn work(&self) -> WorthQueryLiveProjectionRefreshWork {
        self.work
    }

    pub fn impact(&self) -> &crate::domain_installation::WorthQueryImpactDecision {
        &self.impact
    }

    pub fn native_value<'a>(
        &'a self,
        key: &WorthQueryNativeAccessKey,
        row: usize,
    ) -> Result<WorthQueryNativeFieldAccess<'a>, WorthQueryNativeAccessDenial> {
        let Some(layout) = &self.native_access else {
            return Err(WorthQueryNativeAccessLayout::unbound_denial(
                &self.authority,
                key,
            ));
        };
        layout.access(&self.authority, key, row)
    }
}

#[derive(Debug)]
pub enum WorthQueryLiveProjectionRefreshError {
    Impact {
        denial: crate::domain_installation::WorthQueryImpactAdmissionDenial,
        work: WorthQueryLiveProjectionRefreshWork,
        owner_delivery_retained: bool,
    },
    Authority(WorthQueryLiveProjectionRefreshAuthorityStop),
    Runtime {
        error: WorthQueryRuntimeError,
        work: WorthQueryLiveProjectionRefreshWork,
        owner_delivery_retained: bool,
    },
    Conditional {
        kind: WorthQueryProjectionPromotionDenialKind,
        detail: String,
        work: WorthQueryLiveProjectionRefreshWork,
        owner_delivery_retained: bool,
    },
    Projection {
        outcome: Box<WorthQueryProjectionOutcome>,
        work: WorthQueryLiveProjectionRefreshWork,
        owner_delivery_retained: bool,
    },
    NativeAccess {
        denial: WorthQueryNativeAccessDenial,
        work: WorthQueryLiveProjectionRefreshWork,
        owner_delivery_retained: bool,
    },
}

impl WorthQueryLiveProjectionRefreshError {
    pub fn work(&self) -> WorthQueryLiveProjectionRefreshWork {
        match self {
            Self::Impact { work, .. } => *work,
            Self::Authority(stop) => stop.work(),
            Self::Runtime { work, .. }
            | Self::Conditional { work, .. }
            | Self::Projection { work, .. }
            | Self::NativeAccess { work, .. } => *work,
        }
    }

    /// Reports whether exact staged owner evidence remains available for a
    /// corrected retry after this stop.
    pub fn owner_delivery_retained(&self) -> bool {
        match self {
            Self::Impact {
                owner_delivery_retained,
                ..
            }
            | Self::Runtime {
                owner_delivery_retained,
                ..
            }
            | Self::Conditional {
                owner_delivery_retained,
                ..
            }
            | Self::Projection {
                owner_delivery_retained,
                ..
            }
            | Self::NativeAccess {
                owner_delivery_retained,
                ..
            } => *owner_delivery_retained,
            Self::Authority(_) => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveProjectionRefreshAuthorityStop {
    kind: crate::domain_installation::WorthQueryDomainHandleDenialKind,
    work: WorthQueryLiveProjectionRefreshWork,
}

impl WorthQueryLiveProjectionRefreshAuthorityStop {
    pub fn kind(&self) -> crate::domain_installation::WorthQueryDomainHandleDenialKind {
        self.kind
    }

    pub fn work(&self) -> WorthQueryLiveProjectionRefreshWork {
        self.work
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryLiveBoundDomainProjection<D, O, F, L>
{
    pub fn refresh(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError> {
        refresh_source(self.snapshot(), self.managed_handle(), workspace)
    }

    pub(crate) fn refresh_granular_scope(
        &self,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError> {
        refresh_granular_source(
            self.snapshot(),
            self.managed_handle(),
            scope,
            basis,
            workspace,
        )
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryLiveBoundWorkflowProjection<D, O, F, L>
{
    pub fn refresh(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError> {
        refresh_source(self.snapshot(), self.managed_handle(), workspace)
    }
}

pub(super) fn refresh_source<D: 'static, O: 'static, F: 'static, L: BasisOperationLane, S>(
    source: &S,
    handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
    workspace: &mut WorthQueryWorkspace,
) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError>
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    let mut work = WorthQueryLiveProjectionRefreshWork::authority_checked();
    super::source::validate_live_source_authority(source, workspace).map_err(|denial| {
        WorthQueryLiveProjectionRefreshError::Authority(
            WorthQueryLiveProjectionRefreshAuthorityStop {
                kind: denial.kind(),
                work,
            },
        )
    })?;
    let closure = source.semantic_dependency_closure().ok_or_else(||
        crate::domain_installation::WorthQueryImpactAdmissionDenial::new(
            crate::domain_installation::WorthQueryImpactAdmissionDenialKind::DependencyClosureUnavailable,
            crate::domain_installation::WorthQueryImpactCounters::default(),
        )
    ).map_err(|denial| WorthQueryLiveProjectionRefreshError::Impact {
        denial,
        work,
        owner_delivery_retained: false,
    })?;
    work.begin_drain();
    let delivery =
        handle
            .drain(workspace)
            .map_err(|error| WorthQueryLiveProjectionRefreshError::Runtime {
                error,
                work,
                owner_delivery_retained: false,
            })?;
    work.retain_delivery(&delivery);
    let impact = std::sync::Arc::new(
        crate::domain_installation::WorthQueryImpactDecision::from_managed_live_delivery(
            closure, &delivery,
        ),
    );
    work.retain_impact(&impact);
    finish_refresh(source, handle, workspace, delivery, impact, work, None)
}

pub(crate) fn refresh_granular_source<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
    S,
>(
    source: &S,
    handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
    scope: &crate::live::WorthQueryMaintenanceScope,
    basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    workspace: &mut WorthQueryWorkspace,
) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError>
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    let mut work = WorthQueryLiveProjectionRefreshWork::authority_checked();
    super::source::validate_live_source_authority(source, workspace).map_err(|denial| {
        WorthQueryLiveProjectionRefreshError::Authority(
            WorthQueryLiveProjectionRefreshAuthorityStop {
                kind: denial.kind(),
                work,
            },
        )
    })?;
    let closure = source.semantic_dependency_closure().ok_or_else(|| {
        crate::domain_installation::WorthQueryImpactAdmissionDenial::new(
            crate::domain_installation::WorthQueryImpactAdmissionDenialKind::DependencyClosureUnavailable,
            crate::domain_installation::WorthQueryImpactCounters::default(),
        )
    }).map_err(|denial| WorthQueryLiveProjectionRefreshError::Impact {
        denial,
        work,
        owner_delivery_retained: true,
    })?;
    let prepared = projection::prepare_projection(
        source,
        handle,
        workspace,
        Some((scope, basis)),
        work,
        true,
    )?;
    work = prepared.work;
    work.begin_drain();
    let delivery =
        handle
            .drain(workspace)
            .map_err(|error| WorthQueryLiveProjectionRefreshError::Runtime {
                error,
                work,
                owner_delivery_retained: true,
            })?;
    work.retain_delivery(&delivery);
    let impact = std::sync::Arc::new(
        crate::domain_installation::WorthQueryImpactDecision::from_managed_live_delivery(
            closure, &delivery,
        ),
    );
    work.retain_impact(&impact);
    Ok(WorthQueryLiveProjectionRefresh {
        authority: prepared.authority,
        source_rows: prepared.source_rows,
        warnings: prepared.warnings,
        native_access: prepared.native_access,
        delivery,
        work,
        impact,
    })
}

fn finish_refresh<D: 'static, O: 'static, F: 'static, L: BasisOperationLane, S>(
    source: &S,
    handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
    workspace: &mut WorthQueryWorkspace,
    delivery: crate::ordinary::live::WorthQueryManagedLiveDelivery,
    impact: std::sync::Arc<crate::domain_installation::WorthQueryImpactDecision>,
    work: WorthQueryLiveProjectionRefreshWork,
    granular_read: Option<(
        &crate::live::WorthQueryMaintenanceScope,
        &crate::runtime::WorthQueryGranularSourceReadBasis,
    )>,
) -> Result<WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError>
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    let prepared =
        projection::prepare_projection(source, handle, workspace, granular_read, work, false)?;
    Ok(WorthQueryLiveProjectionRefresh {
        authority: prepared.authority,
        source_rows: prepared.source_rows,
        warnings: prepared.warnings,
        native_access: prepared.native_access,
        delivery,
        work: prepared.work,
        impact,
    })
}
