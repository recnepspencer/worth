use crate::access::shape::{AccessAuthorityPosture, AccessShape};
use crate::access::AdmittedAccessIntent;
use crate::artifact_family::AdmittedPhysicalArtifactFamily;
use crate::catalog::{declare_authority_role, AuthorityRole};
use crate::keyspace::AdmittedPhysicalKeyDomain;
use crate::maintenance::IndexMaintenanceMode;
use crate::materialization::AdmittedLayoutMaterialization;
use crate::strategy::registry::{
    layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
    LayoutStrategyRegistrySnapshot,
};
use crate::strategy::LayoutStrategyFamily;

use super::super::denial::SelectionCandidateRejection;
use super::super::selection_basis::{PlanningCapabilityGrant, SelectionCandidateEligibility};
use super::{
    classify_candidate_operation, CandidateStrategyFamily, EligibleLsmOperation,
    SelectionCandidateAudit, SelectionCandidateOutcome,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::planning) struct PlanningAlternative {
    snapshot: LayoutStrategyRegistrySnapshot,
    operation: EligibleLsmOperation,
    audit: SelectionCandidateAudit,
}

impl PlanningAlternative {
    pub(in crate::planning) const fn snapshot(&self) -> &LayoutStrategyRegistrySnapshot {
        &self.snapshot
    }

    pub(in crate::planning) const fn operation(&self) -> EligibleLsmOperation {
        self.operation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::planning) struct PlanningAlternativeSet {
    selected: Option<PlanningAlternative>,
    primary_audit: SelectionCandidateAudit,
    secondary_audit: SelectionCandidateAudit,
}

impl PlanningAlternativeSet {
    pub(in crate::planning) fn derive(
        family: AdmittedPhysicalArtifactFamily,
        key_domain: AdmittedPhysicalKeyDomain,
        materialization: Option<AdmittedLayoutMaterialization>,
        intent: AdmittedAccessIntent,
    ) -> Self {
        let role = declare_authority_role(family.classification()).role();
        let btree = derive_candidate(
            family,
            key_domain,
            materialization.as_ref(),
            intent,
            CandidateStrategyFamily::BTree,
            role,
        );
        let lsm = derive_candidate(
            family,
            key_domain,
            materialization.as_ref(),
            intent,
            CandidateStrategyFamily::Lsm,
            role,
        );

        let selected = match key_domain.domain() {
            crate::keyspace::PhysicalKeyDomain::PageAddressKey
            | crate::keyspace::PhysicalKeyDomain::SegmentAddressKey
            | crate::keyspace::PhysicalKeyDomain::ExtentAddressKey
            | crate::keyspace::PhysicalKeyDomain::PhysicalReferenceKey => btree.0,
            crate::keyspace::PhysicalKeyDomain::WalRecordKey
            | crate::keyspace::PhysicalKeyDomain::BlobIdentityKey => lsm.0,
            crate::keyspace::PhysicalKeyDomain::RootManifestKey => None,
        };

        Self {
            selected,
            primary_audit: btree.1,
            secondary_audit: lsm.1,
        }
    }

    pub(in crate::planning) const fn selected(&self) -> Option<&PlanningAlternative> {
        self.selected.as_ref()
    }

    pub(in crate::planning) const fn primary_audit(&self) -> &SelectionCandidateAudit {
        &self.primary_audit
    }

    pub(in crate::planning) const fn secondary_audit(&self) -> &SelectionCandidateAudit {
        &self.secondary_audit
    }
}

fn derive_candidate(
    admitted_family: AdmittedPhysicalArtifactFamily,
    key_domain: AdmittedPhysicalKeyDomain,
    materialization: Option<&AdmittedLayoutMaterialization>,
    intent: AdmittedAccessIntent,
    candidate_family: CandidateStrategyFamily,
    authority_role: AuthorityRole,
) -> (Option<PlanningAlternative>, SelectionCandidateAudit) {
    let strategy_family = candidate_family.strategy_family();
    let operation = match classify_candidate_operation(candidate_family, intent.shape()) {
        Ok(operation) => operation,
        Err(rejection) => {
            return (
                None,
                SelectionCandidateAudit::new(
                    strategy_family,
                    authority_role,
                    SelectionCandidateOutcome::Rejected(rejection),
                ),
            );
        }
    };
    let request = build_request(
        admitted_family,
        key_domain,
        materialization,
        intent,
        strategy_family,
    );
    match layout_admission_registry().admit(request).into_result() {
        Ok(snapshot) => {
            admit_candidate(snapshot, operation, intent, strategy_family, authority_role)
        }
        Err(denial) => (
            None,
            SelectionCandidateAudit::new(
                strategy_family,
                authority_role,
                SelectionCandidateOutcome::Rejected(SelectionCandidateRejection::RegistryDenied(
                    Box::new(denial),
                )),
            ),
        ),
    }
}

fn admit_candidate(
    snapshot: LayoutStrategyRegistrySnapshot,
    operation: EligibleLsmOperation,
    intent: AdmittedAccessIntent,
    strategy_family: LayoutStrategyFamily,
    authority_role: AuthorityRole,
) -> (Option<PlanningAlternative>, SelectionCandidateAudit) {
    let Some(planned_counter_envelope) = crate::strategy::planned_counter_envelope_for(
        snapshot.admitted_strategy().family(),
        intent.detail(),
    ) else {
        return (
            None,
            SelectionCandidateAudit::new(
                strategy_family,
                authority_role,
                SelectionCandidateOutcome::Rejected(
                    SelectionCandidateRejection::MissingPlannedCounterEnvelope,
                ),
            ),
        );
    };
    let audit = SelectionCandidateAudit::new(
        strategy_family,
        authority_role,
        SelectionCandidateOutcome::Eligible(SelectionCandidateEligibility::RegistryAdmitted {
            granted_capability: planning_capability(snapshot.granted_capability()),
            planned_counter_envelope,
        }),
    );
    (
        Some(PlanningAlternative {
            snapshot,
            operation,
            audit: audit.clone(),
        }),
        audit,
    )
}

const fn planning_capability(
    capability: crate::strategy::registry::LayoutStrategyCapability,
) -> PlanningCapabilityGrant {
    match capability {
        crate::strategy::registry::LayoutStrategyCapability::PointLookup => {
            PlanningCapabilityGrant::PointLookup
        }
        crate::strategy::registry::LayoutStrategyCapability::OrderedRange => {
            PlanningCapabilityGrant::OrderedRange
        }
        crate::strategy::registry::LayoutStrategyCapability::PrefixTraversal => {
            PlanningCapabilityGrant::PrefixTraversal
        }
        crate::strategy::registry::LayoutStrategyCapability::BlobStreaming => {
            PlanningCapabilityGrant::BlobStreaming
        }
        crate::strategy::registry::LayoutStrategyCapability::ExactScan => {
            PlanningCapabilityGrant::ExactScan
        }
    }
}

fn build_request(
    family_authority: AdmittedPhysicalArtifactFamily,
    key_domain: AdmittedPhysicalKeyDomain,
    materialization: Option<&AdmittedLayoutMaterialization>,
    intent: AdmittedAccessIntent,
    family: LayoutStrategyFamily,
) -> LayoutAdmissionRequest {
    let mut request = LayoutAdmissionRequest::from_admitted(
        family_authority,
        key_domain,
        family,
        requested_capability(intent.shape()),
        intent.lane().admitted_lane(),
    )
    .under_maintenance_mode(maintenance_mode_for(intent));

    if let Some(mutation_shape) = intent.mutation_shape() {
        request = request.for_mutation_shape(mutation_shape);
    }
    if let Some(materialization) = materialization {
        request = request.require_exact_materialization(materialization.coverage().clone());
    } else if intent.authority_posture() == AccessAuthorityPosture::ExactMaterialized {
        request = request.require_exact_readiness();
    }

    request
}

const fn maintenance_mode_for(shape: AdmittedAccessIntent) -> IndexMaintenanceMode {
    use crate::access::shape::AccessLaneClassification;

    match shape.lane() {
        AccessLaneClassification::Foreground => IndexMaintenanceMode::SynchronousExact,
        AccessLaneClassification::Maintenance => match shape.shape() {
            AccessShape::RebuildRead => IndexMaintenanceMode::RebuildOnly,
            _ => IndexMaintenanceMode::AsynchronousLagged,
        },
        AccessLaneClassification::Verifier => IndexMaintenanceMode::VerifierOnly,
        AccessLaneClassification::Terminal => IndexMaintenanceMode::AdvisoryOnly,
    }
}

const fn requested_capability(shape: AccessShape) -> LayoutRequestedCapability {
    match shape {
        AccessShape::PointLookup
        | AccessShape::BatchPointLookup
        | AccessShape::SortedBatchLookup
        | AccessShape::Append => LayoutRequestedCapability::PointLookup,
        AccessShape::RangeLookup
        | AccessShape::MultiRangeLookup
        | AccessShape::CoalescedPageRead => LayoutRequestedCapability::OrderedRange,
        AccessShape::PrefixLookup | AccessShape::GroupedPrefixLookup => {
            LayoutRequestedCapability::PrefixTraversal
        }
        AccessShape::ChunkTreeWalk
        | AccessShape::StreamingRead
        | AccessShape::StreamingContinuationRead => LayoutRequestedCapability::BlobStreaming,
        _ => LayoutRequestedCapability::ExactScan,
    }
}
