use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;

use super::super::projection_lifecycle::{
    admit_projection_promotion_core, WorthQueryProjectionCoreStop,
    WorthQueryProjectionLifecycleSource,
};
use super::super::{WorthQueryCurrentDomainProjection, WorthQueryLiveBoundDomainProjection};
use super::{WorthQueryAdmittedProjectionSharing, WorthQuerySharedLiveProjectionLease};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProjectionSharingDenialKind {
    ConsumerSupport,
    CandidateNotCurrent,
    ExecutionSharing,
    DependencyClosure,
    ProjectionContract,
    NativeProjectionLayout,
    InstalledRead,
    LiveOwnerRegistration,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryProjectionSharingCounters {
    pub support_posture_checks: usize,
    pub candidate_preflight_checks: usize,
    pub subject_preflight: super::super::WorthQueryProjectionPromotionCounters,
    pub candidate_preflight: super::super::WorthQueryProjectionPromotionCounters,
    pub compatibility: crate::domain_installation::WorthQueryCompatibilityCounters,
    pub compatibility_checks: usize,
    pub closure_comparisons: usize,
    pub closure_readmissions: usize,
    pub dependency_edges_compared: usize,
    pub dependency_edges_readmitted: usize,
    pub projection_contract_checks: usize,
    pub native_layout_checks: usize,
    pub installed_read_checks: usize,
    pub owner_registrations: usize,
    pub lease_issues: usize,
    pub unrelated_registry_scans: usize,
}

pub struct WorthQuerySharedLiveProjectionPair<D, O, F, L: BasisOperationLane> {
    subject: WorthQuerySharedLiveProjectionLease<D, O, F, L>,
    candidate: WorthQuerySharedLiveProjectionLease<D, O, F, L>,
    counters: WorthQueryProjectionSharingCounters,
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySharedLiveProjectionPair<D, O, F, L> {
    pub(super) fn new(
        subject: WorthQuerySharedLiveProjectionLease<D, O, F, L>,
        candidate: WorthQuerySharedLiveProjectionLease<D, O, F, L>,
        counters: WorthQueryProjectionSharingCounters,
    ) -> Self {
        Self {
            subject,
            candidate,
            counters,
        }
    }

    pub fn into_leases(
        self,
    ) -> (
        WorthQuerySharedLiveProjectionLease<D, O, F, L>,
        WorthQuerySharedLiveProjectionLease<D, O, F, L>,
    ) {
        (self.subject, self.candidate)
    }

    pub const fn counters(&self) -> WorthQueryProjectionSharingCounters {
        self.counters
    }
}

#[must_use = "sharing stops retain both exact lifecycle inputs"]
pub enum WorthQueryProjectionSharingOutcome<D, O, F, L: BasisOperationLane> {
    Shared(WorthQuerySharedLiveProjectionPair<D, O, F, L>),
    Stopped(WorthQueryProjectionSharingStop<D, O, F, L>),
}

pub struct WorthQueryProjectionSharingStop<D, O, F, L: BasisOperationLane> {
    live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
    kind: WorthQueryProjectionSharingDenialKind,
    detail: String,
    counters: WorthQueryProjectionSharingCounters,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryProjectionSharingStop<D, O, F, L> {
    pub const fn kind(&self) -> WorthQueryProjectionSharingDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryProjectionSharingCounters {
        self.counters
    }

    pub fn into_inputs(
        self,
    ) -> (
        WorthQueryLiveBoundDomainProjection<D, O, F, L>,
        WorthQueryCurrentDomainProjection<D, O, F, L>,
    ) {
        (self.live, self.candidate)
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryLiveBoundDomainProjection<D, O, F, L>
{
    pub fn share_with(
        self,
        candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionSharingOutcome<D, O, F, L> {
        let mut counters = WorthQueryProjectionSharingCounters::default();
        let candidate_read = match admit_projection_promotion_core(
            candidate.snapshot(),
            candidate.lifecycle_basis(),
            workspace,
        ) {
            Ok(admitted) => {
                counters.candidate_preflight_checks = 1;
                counters.candidate_preflight = admitted.counters;
                admitted.read
            }
            Err(stop) => {
                return stopped(
                    self,
                    candidate,
                    WorthQueryProjectionSharingDenialKind::CandidateNotCurrent,
                    core_stop_detail(stop),
                    counters,
                )
            }
        };
        match admit_projection_promotion_core(self.snapshot(), self.lifecycle_basis(), workspace) {
            Ok(admitted) => counters.subject_preflight = admitted.counters,
            Err(stop) => {
                return stopped(
                    self,
                    candidate,
                    WorthQueryProjectionSharingDenialKind::CandidateNotCurrent,
                    core_stop_detail(stop),
                    counters,
                )
            }
        }
        counters.support_posture_checks = 2;
        let subject_supports_sharing = self.snapshot().consumer_contract().support_posture(
            crate::domain_installation::WorthQueryConsumerSupportDimension::Sharing,
        )
            == crate::domain_installation::WorthQueryConsumerSupportPosture::Supported;
        let candidate_supports_sharing = candidate.snapshot().consumer_contract().support_posture(
            crate::domain_installation::WorthQueryConsumerSupportDimension::Sharing,
        )
            == crate::domain_installation::WorthQueryConsumerSupportPosture::Supported;
        if !subject_supports_sharing || !candidate_supports_sharing {
            return stopped(
                self,
                candidate,
                WorthQueryProjectionSharingDenialKind::ConsumerSupport,
                "shared execution requires Supported consumer sharing posture",
                counters,
            );
        }

        counters.projection_contract_checks += 1;
        if !self
            .snapshot()
            .consumer_contract()
            .shares_execution_projection_with(candidate.snapshot().consumer_contract())
        {
            return stopped(
                self,
                candidate,
                WorthQueryProjectionSharingDenialKind::ProjectionContract,
                "consumer projection contracts do not share exact structural meaning",
                counters,
            );
        }
        counters.native_layout_checks += 1;
        if !native_layouts_match(self.snapshot(), candidate.snapshot()) {
            return stopped(
                self,
                candidate,
                WorthQueryProjectionSharingDenialKind::NativeProjectionLayout,
                "native projection layouts differ structurally",
                counters,
            );
        }
        counters.installed_read_checks += 1;
        if !installed_reads_match(self.snapshot(), &candidate_read) {
            return stopped(
                self,
                candidate,
                WorthQueryProjectionSharingDenialKind::InstalledRead,
                "candidate installed read differs from the live owner's structural read",
                counters,
            );
        }

        let sharing = match self
            .snapshot()
            .bound_operation()
            .execution_sharing_with(candidate.snapshot().bound_operation())
        {
            Ok(sharing) => sharing,
            Err(denial) => {
                return stopped(
                    self,
                    candidate,
                    WorthQueryProjectionSharingDenialKind::ExecutionSharing,
                    format!("{:?}", denial.kind()),
                    counters,
                )
            }
        };
        counters.compatibility = sharing.counters();
        counters.compatibility_checks = compatibility_work(counters.compatibility);
        let subject_closure = self.snapshot().semantic_aspect_dependency_closure();
        let candidate_closure = candidate.snapshot().semantic_aspect_dependency_closure();
        let reuse = match sharing.admit_dependency_closure_reuse(
            self.snapshot().bound_operation(),
            candidate.snapshot().bound_operation(),
            subject_closure,
            candidate_closure,
        ) {
            Ok(reuse) => reuse,
            Err(denial) => {
                return stopped(
                    self,
                    candidate,
                    WorthQueryProjectionSharingDenialKind::DependencyClosure,
                    format!("{denial:?}"),
                    counters,
                )
            }
        };
        counters.closure_comparisons = 1;
        counters.dependency_edges_compared = reuse.dependency_count();
        let reuse = match reuse.readmit_for_pair(
            self.snapshot().bound_operation(),
            candidate.snapshot().bound_operation(),
            subject_closure,
            candidate_closure,
        ) {
            Ok(reuse) => reuse,
            Err(denial) => {
                return stopped(
                    self,
                    candidate,
                    WorthQueryProjectionSharingDenialKind::DependencyClosure,
                    format!("{denial:?}"),
                    counters,
                )
            }
        };
        counters.closure_readmissions = 1;
        counters.dependency_edges_readmitted = reuse.dependency_count();
        let admitted = WorthQueryAdmittedProjectionSharing::equivalent(
            reuse,
            self.snapshot(),
            candidate.snapshot(),
        );

        super::pair_registration::register_shared_pair(
            self, candidate, admitted, workspace, counters,
        )
    }
}

fn compatibility_work(
    counters: crate::domain_installation::WorthQueryCompatibilityCounters,
) -> usize {
    counters.canonical_comparisons
        + counters.portable_contract_comparisons
        + counters.portable_variable_items_submitted
        + counters.portable_conditional_nodes_submitted
        + counters.retained_authority_checks
        + counters.required_domain_rebind_receipts_inspected
        + counters.conditional_lowerings_compared
        + counters.conditional_foundational_comparisons
        + counters.conditional_liveness_checks
        + counters.conditional_correspondences_inspected
        + counters.conditional_targets_inspected
        + counters.conditional_provider_roles_inspected
        + counters.conditional_signal_semantic_dimensions_inspected
        + counters.conditional_signal_affinity_dimensions_inspected
        + counters.conditional_bridge_affinity_dimensions_inspected
        + counters.lower_runtime_contacts
        + counters.execution_calls
        + counters.maintenance_calls
}

fn native_layouts_match<D, O, F, L: BasisOperationLane>(
    subject: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
    candidate: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
) -> bool {
    match (
        subject.native_access_layout(),
        candidate.native_access_layout(),
    ) {
        (Some(subject), Some(candidate)) => subject.shares_execution_projection_with(candidate),
        (None, None) => true,
        _ => false,
    }
}

fn installed_reads_match<D, O, F, L: BasisOperationLane>(
    subject: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
    candidate: &crate::ordinary::read::WorthQueryReadDeclaration,
) -> bool {
    let mut checks = 0;
    subject
        .installed_read(&mut checks)
        .is_some_and(|subject| subject == *candidate)
}

fn core_stop_detail(stop: WorthQueryProjectionCoreStop) -> &'static str {
    match stop {
        WorthQueryProjectionCoreStop::Stale(_) => "candidate installation is stale",
        WorthQueryProjectionCoreStop::RebindRequired(_) => "candidate requires explicit rebind",
        WorthQueryProjectionCoreStop::AuthorityRevalidationRequired(_) => {
            "candidate authority requires revalidation"
        }
        WorthQueryProjectionCoreStop::Denied { detail, .. } => detail,
    }
}

pub(super) fn stopped<D, O, F, L: BasisOperationLane>(
    live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
    kind: WorthQueryProjectionSharingDenialKind,
    detail: impl Into<String>,
    counters: WorthQueryProjectionSharingCounters,
) -> WorthQueryProjectionSharingOutcome<D, O, F, L> {
    WorthQueryProjectionSharingOutcome::Stopped(WorthQueryProjectionSharingStop {
        live,
        candidate,
        kind,
        detail: detail.into(),
        counters,
    })
}
#[cfg(test)]
#[path = "admission_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
