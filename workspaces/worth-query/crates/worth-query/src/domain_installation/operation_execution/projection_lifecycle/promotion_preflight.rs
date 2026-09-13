use crate::basis_lifecycle::{
    BasisAuthorityPosture, BasisFamily, BasisLifecyclePosture, BasisOperationLane,
    BasisScopePosture, BasisVisibilityPosture,
};
use crate::domain_installation::{
    WorthQueryConsumerSupportDimension, WorthQueryConsumerSupportPosture,
};
use crate::runtime::WorthQueryWorkspace;

use super::{
    WorthQueryCurrentDomainProjection, WorthQueryProjectionPromotionCounters,
    WorthQueryProjectionPromotionDenialKind, WorthQueryProjectionPromotionOutcome,
    WorthQueryProjectionPromotionStop,
};

pub(super) struct WorthQueryAdmittedProjectionPromotion<D, O, F, L: BasisOperationLane> {
    pub(super) current: WorthQueryCurrentDomainProjection<D, O, F, L>,
    pub(super) read: crate::ordinary::read::WorthQueryReadDeclaration,
    pub(in crate::domain_installation::operation_execution) counters:
        WorthQueryProjectionPromotionCounters,
}

pub(super) enum WorthQueryProjectionPreflightOutcome<D, O, F, L: BasisOperationLane> {
    Admitted(Box<WorthQueryAdmittedProjectionPromotion<D, O, F, L>>),
    Stopped(Box<WorthQueryProjectionPromotionOutcome<D, O, F, L>>),
}

pub(in crate::domain_installation::operation_execution) struct WorthQueryProjectionCoreAdmission {
    pub(in crate::domain_installation::operation_execution) read:
        crate::ordinary::read::WorthQueryReadDeclaration,
    pub(in crate::domain_installation::operation_execution) counters:
        WorthQueryProjectionPromotionCounters,
}

pub(in crate::domain_installation::operation_execution) enum WorthQueryProjectionCoreStop {
    Stale(WorthQueryProjectionPromotionCounters),
    RebindRequired(WorthQueryProjectionPromotionCounters),
    AuthorityRevalidationRequired(WorthQueryProjectionPromotionCounters),
    Denied {
        kind: WorthQueryProjectionPromotionDenialKind,
        detail: &'static str,
        counters: WorthQueryProjectionPromotionCounters,
    },
}

pub(super) fn admit_projection_promotion<D: 'static, O, F, L: BasisOperationLane>(
    current: WorthQueryCurrentDomainProjection<D, O, F, L>,
    workspace: &WorthQueryWorkspace,
) -> WorthQueryProjectionPreflightOutcome<D, O, F, L> {
    match admit_projection_promotion_core(&current.settled, current.lifecycle_basis(), workspace) {
        Ok(admitted) => WorthQueryProjectionPreflightOutcome::Admitted(Box::new(
            WorthQueryAdmittedProjectionPromotion {
                current,
                read: admitted.read,
                counters: admitted.counters,
            },
        )),
        Err(stop) => {
            WorthQueryProjectionPreflightOutcome::Stopped(Box::new(map_core_stop(current, stop)))
        }
    }
}

pub(in crate::domain_installation::operation_execution) fn admit_projection_promotion_core<
    D: 'static,
    O,
    F,
    L: BasisOperationLane,
    S: super::source::WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    source: &S,
    lifecycle_basis: &super::states::WorthQueryProjectionLifecycleBasis<L>,
    workspace: &WorthQueryWorkspace,
) -> Result<WorthQueryProjectionCoreAdmission, WorthQueryProjectionCoreStop> {
    let mut counters = WorthQueryProjectionPromotionCounters::default();
    let bound = source.bound_operation();
    counters.retained_authority_checks += 1;
    if bound.operation().domain_authority().runtime_authority()
        != workspace.runtime_authority_identity()
    {
        return denied(
            WorthQueryProjectionPromotionDenialKind::ForeignRuntime,
            "projection belongs to a different Query runtime authority",
            counters,
        );
    }
    if !lifecycle_basis.binds(source, &mut counters.retained_authority_checks) {
        return denied(
            WorthQueryProjectionPromotionDenialKind::BoundAuthorityMismatch,
            "lifecycle proof no longer binds the settled projection authority",
            counters,
        );
    }
    counters.retained_authority_checks += 1;
    let witness =
        crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
            std::sync::Arc::clone(bound.operation().domain_authority()),
        );
    if let Err(denial) = workspace.validate_installed_domain_witness::<D>(&witness) {
        use crate::domain_installation::WorthQueryDomainHandleDenialKind as Kind;
        return Err(match denial.kind() {
            Kind::StaleInstallationGeneration => WorthQueryProjectionCoreStop::Stale(counters),
            Kind::PackageIdentityChanged => WorthQueryProjectionCoreStop::RebindRequired(counters),
            Kind::DomainNotInstalled => WorthQueryProjectionCoreStop::Denied {
                kind: WorthQueryProjectionPromotionDenialKind::DomainNotInstalled,
                detail: "the projection domain is not installed in this workspace registry",
                counters,
            },
            Kind::ForeignRuntime => WorthQueryProjectionCoreStop::Denied {
                kind: WorthQueryProjectionPromotionDenialKind::ForeignRuntime,
                detail: "projection belongs to a different Query runtime authority",
                counters,
            },
        });
    }
    if !is_live_current_basis(bound.basis().normalized(), &mut counters.basis_checks) {
        return Err(WorthQueryProjectionCoreStop::AuthorityRevalidationRequired(
            counters,
        ));
    }
    if !consumer_authority_is_exact(source, &mut counters.retained_authority_checks) {
        return denied(
            WorthQueryProjectionPromotionDenialKind::BoundAuthorityMismatch,
            "consumer contract no longer binds the settled projection capability",
            counters,
        );
    }
    if !live_support_is_admitted(source, &mut counters.support_checks) {
        return denied(
            WorthQueryProjectionPromotionDenialKind::LiveSupportUnavailable,
            "installed consumer support does not admit this live projection",
            counters,
        );
    }
    let Some(read) = source.installed_read(&mut counters.installed_read_checks) else {
        return denied(
            WorthQueryProjectionPromotionDenialKind::InstalledReadUnavailable,
            "projection has no exact installation-validated read",
            counters,
        );
    };
    for node in bound
        .conditional_nodes()
        .iter()
        .filter(|node| source.admits_conditional_location(&node.location))
    {
        counters.conditional_lowerings_checked += 1;
        if node.lowering.admit_live_conditional_lowering().is_err() {
            return denied(
                WorthQueryProjectionPromotionDenialKind::ConditionalLoweringNotLive,
                "an installed conditional lowering is no longer live",
                counters,
            );
        }
    }
    Ok(WorthQueryProjectionCoreAdmission { read, counters })
}

fn map_core_stop<D, O, F, L: BasisOperationLane>(
    current: WorthQueryCurrentDomainProjection<D, O, F, L>,
    stop: WorthQueryProjectionCoreStop,
) -> WorthQueryProjectionPromotionOutcome<D, O, F, L> {
    match stop {
        WorthQueryProjectionCoreStop::Stale(counters) => {
            WorthQueryProjectionPromotionOutcome::Stale(current.into_stale(counters))
        }
        WorthQueryProjectionCoreStop::RebindRequired(counters) => {
            WorthQueryProjectionPromotionOutcome::RebindRequired(
                current.into_rebind_required(counters),
            )
        }
        WorthQueryProjectionCoreStop::AuthorityRevalidationRequired(counters) => {
            WorthQueryProjectionPromotionOutcome::AuthorityRevalidationRequired(
                current.into_authority_revalidation(counters),
            )
        }
        WorthQueryProjectionCoreStop::Denied {
            kind,
            detail,
            counters,
        } => WorthQueryProjectionPromotionOutcome::Denied(WorthQueryProjectionPromotionStop::new(
            current, kind, detail, counters,
        )),
    }
}

fn denied<T>(
    kind: WorthQueryProjectionPromotionDenialKind,
    detail: &'static str,
    counters: WorthQueryProjectionPromotionCounters,
) -> Result<T, WorthQueryProjectionCoreStop> {
    Err(WorthQueryProjectionCoreStop::Denied {
        kind,
        detail,
        counters,
    })
}

pub(super) fn is_live_current_basis(
    basis: &crate::basis_lifecycle::NormalizedBasisIntent,
    checks: &mut usize,
) -> bool {
    checked(checks, basis.family() == BasisFamily::CurrentHead)
        && checked(checks, basis.authority() == BasisAuthorityPosture::Runtime)
        && checked(checks, basis.scope() == BasisScopePosture::Global)
        && checked(checks, basis.visibility() == BasisVisibilityPosture::Full)
        && checked(checks, basis.lifecycle() == BasisLifecyclePosture::Current)
}

fn consumer_authority_is_exact<
    D,
    O,
    F,
    L: BasisOperationLane,
    S: super::source::WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    source: &S,
    checks: &mut usize,
) -> bool {
    let bound = source.bound_operation();
    let consumer = source.consumer_contract();
    checked(
        checks,
        consumer.binds_capability(bound.capability_identity()),
    ) && checked(
        checks,
        consumer.binding_identity() == bound.binding_identity(),
    ) && checked(
        checks,
        consumer.operation_identity() == bound.definition().identity(),
    ) && checked(
        checks,
        consumer.installation_generation() == bound.operation().installation_generation(),
    ) && checked(
        checks,
        consumer.basis_identity() == bound.basis().capability_digest(),
    )
}

fn live_support_is_admitted<
    D,
    O,
    F,
    L: BasisOperationLane,
    S: super::source::WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    source: &S,
    checks: &mut usize,
) -> bool {
    let consumer = source.consumer_contract();
    if !supported(consumer, WorthQueryConsumerSupportDimension::Live, checks) {
        return false;
    }
    if !source
        .bound_operation()
        .conditional_nodes()
        .iter()
        .any(|node| source.admits_conditional_location(&node.location))
    {
        return true;
    }
    [
        WorthQueryConsumerSupportDimension::ConditionalEvaluation,
        WorthQueryConsumerSupportDimension::ConditionalComparator,
        WorthQueryConsumerSupportDimension::ConditionalTrigger,
        WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
    ]
    .into_iter()
    .all(|dimension| supported(consumer, dimension, checks))
}

fn supported<D, O, F, L: BasisOperationLane>(
    consumer: &crate::domain_installation::WorthQueryConsumerProjectionContract<D, O, F, L>,
    dimension: WorthQueryConsumerSupportDimension,
    checks: &mut usize,
) -> bool {
    checked(
        checks,
        consumer.support_posture(dimension) == WorthQueryConsumerSupportPosture::Supported,
    )
}

fn checked(checks: &mut usize, matches: bool) -> bool {
    *checks += 1;
    matches
}
