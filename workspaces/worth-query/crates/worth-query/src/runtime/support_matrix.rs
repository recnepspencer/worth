use crate::application::{
    WorthQueryMilestoneNineSevenDerivedClosure, WorthQuerySharedReadPinningCertification,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryRuntime, WorthQueryRuntimeBackendPosture,
    WorthQueryRuntimeDownstreamDeliveryContract, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupportStatus, WorthQueryRuntimeFamilyTeachingPosture,
    WorthQueryRuntimePublicApiContract,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicSupportMatrixRow {
    surface: String,
    facade_family: Option<WorthQueryRuntimeFacadeFamily>,
    status: WorthQueryRuntimeFamilySupportStatus,
    teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
    owner_milestone: String,
    extension_rule: String,
    parallel_api_forbidden: bool,
    admission_fail_closed: bool,
    support_contract_digest: Option<String>,
    row_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicSupportMatrixRow {
    fn new(
        surface: impl Into<String>,
        facade_family: Option<WorthQueryRuntimeFacadeFamily>,
        status: WorthQueryRuntimeFamilySupportStatus,
        teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
        owner_milestone: impl Into<String>,
        extension_rule: impl Into<String>,
        parallel_api_forbidden: bool,
        admission_fail_closed: bool,
        support_contract_digest: Option<impl Into<String>>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let surface = surface.into();
        let owner_milestone = owner_milestone.into();
        let extension_rule = extension_rule.into();
        let support_contract_digest = support_contract_digest.map(Into::into);
        let row_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrixRow)
                .field_shape(WorthQueryEvidenceTag::new("surface"), surface.clone())
                .field_shape(
                    WorthQueryEvidenceTag::new("facade_family"),
                    facade_family
                        .map(WorthQueryRuntimeFacadeFamily::as_str)
                        .unwrap_or("matrix-only"),
                )
                .field_shape(WorthQueryEvidenceTag::new("status"), status.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("teaching_posture"),
                    teaching_posture.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("owner_milestone"),
                    owner_milestone.clone(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("extension_rule"),
                    extension_rule.clone(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("parallel_api_forbidden"),
                    parallel_api_forbidden,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("admission_fail_closed"),
                    admission_fail_closed,
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("support_contract_digest"),
                    support_contract_digest.as_deref(),
                )
                .seal()?;
        Ok(Self {
            surface,
            facade_family,
            status,
            teaching_posture,
            owner_milestone,
            extension_rule,
            parallel_api_forbidden,
            admission_fail_closed,
            support_contract_digest,
            row_digest,
        })
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn facade_family(&self) -> Option<WorthQueryRuntimeFacadeFamily> {
        self.facade_family
    }

    pub fn status(&self) -> WorthQueryRuntimeFamilySupportStatus {
        self.status
    }

    pub fn teaching_posture(&self) -> WorthQueryRuntimeFamilyTeachingPosture {
        self.teaching_posture
    }

    pub fn ordinary_downstream_dx(&self) -> bool {
        self.teaching_posture == WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx
    }

    pub fn owner_milestone(&self) -> &str {
        &self.owner_milestone
    }

    pub fn extension_rule(&self) -> &str {
        &self.extension_rule
    }

    pub fn parallel_api_forbidden(&self) -> bool {
        self.parallel_api_forbidden
    }

    pub fn admission_fail_closed(&self) -> bool {
        self.admission_fail_closed
    }

    pub fn support_contract_digest(&self) -> Option<&str> {
        self.support_contract_digest.as_deref()
    }

    pub fn row_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicSupportMatrix {
    backend_posture: WorthQueryRuntimeBackendPosture,
    rows: Vec<WorthQueryRuntimePublicSupportMatrixRow>,
    stable_row_count: usize,
    deferred_row_count: usize,
    unsupported_row_count: usize,
    fail_closed_row_count: usize,
    parallel_api_forbidden_row_count: usize,
    matrix_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicSupportMatrix {
    pub fn from_public_api_contract(
        contract: &WorthQueryRuntimePublicApiContract,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut rows = contract
            .families()
            .iter()
            .map(|family| {
                let status = family.status();
                let facade_family = family.family();
                WorthQueryRuntimePublicSupportMatrixRow::new(
                    facade_family.as_str(),
                    Some(facade_family),
                    status,
                    family.teaching_posture(),
                    family.owner_closure(),
                    family.extension_rule(),
                    family.parallel_api_forbidden(),
                    family.admission_fail_closed(),
                    Some(family.contract_digest().to_string()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "authoritative-mutation-evidence-certification",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Runtime Authoritative Mutation Evidence Gate",
            "must-extend-target-binding-naming-continuity-causality-provenance-contract",
            true,
            false,
            Some(
                WorthQueryRuntime::public_authoritative_mutation_evidence_support_for_posture(
                    contract.backend_posture(),
                )
                .support_digest()
                .to_string(),
            ),
        )?);

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "temporal-async-certification",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Milestone 9.4",
            "must-extend-stabilized-handle-state-lane-aspect-inspection-facade",
            true,
            false,
            None::<String>,
        )?);

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "temporal-async-remask",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Milestone 9.4",
            "must-remask-before-runtime-delivery-state-and-inspection-projection",
            true,
            true,
            None::<String>,
        )?);

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "downstream-delivery-contract",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Milestone 9.4",
            "must-project-runtime-backed-delivery-basis-resume-and-support-posture-through-one-query-owned-contract",
            true,
            false,
            Some(
                WorthQueryRuntimeDownstreamDeliveryContract::from_backend_posture(
                    contract.backend_posture(),
                )?
                .contract_for_reporting()
                .to_string(),
            ),
        )?);

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "shared-read-pinning-boundary-closure",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Milestone 9.7 Phase 13",
            "must-close-shared-read-context-pinning-boundary-before-journal-and-certification-phases",
            true,
            true,
            Some(shared_read_pinning_boundary_closure_contract_digest()),
        )?);

        rows.push(WorthQueryRuntimePublicSupportMatrixRow::new(
            "milestone-9.7-derived-closure-posture",
            None,
            WorthQueryRuntimeFamilySupportStatus::Supported,
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            "Milestone 9.7 Phase 18",
            "must-derive-milestone-closure-from-phase-local-postures",
            true,
            true,
            Some(milestone_nine_seven_derived_closure_contract_digest()),
        )?);

        let stable_row_count = rows
            .iter()
            .filter(|row| row.status() == WorthQueryRuntimeFamilySupportStatus::Supported)
            .count();
        let deferred_row_count = rows
            .iter()
            .filter(|row| row.status() == WorthQueryRuntimeFamilySupportStatus::DeferredDebt)
            .count();
        let unsupported_row_count = rows
            .iter()
            .filter(|row| row.status() == WorthQueryRuntimeFamilySupportStatus::Unsupported)
            .count();
        let fail_closed_row_count = rows
            .iter()
            .filter(|row| row.admission_fail_closed())
            .count();
        let parallel_api_forbidden_row_count = rows
            .iter()
            .filter(|row| row.parallel_api_forbidden())
            .count();
        let matrix_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrix)
                .field_shape(
                    WorthQueryEvidenceTag::new("backend_posture"),
                    contract.backend_posture().as_str(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("stable_row_count"),
                    stable_row_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("deferred_row_count"),
                    deferred_row_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("unsupported_row_count"),
                    unsupported_row_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("fail_closed_row_count"),
                    fail_closed_row_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("parallel_api_forbidden_row_count"),
                    parallel_api_forbidden_row_count,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("row_digest"),
                    rows.iter().map(|row| row.row_digest().as_str()),
                )
                .seal()?;
        Ok(Self {
            backend_posture: contract.backend_posture(),
            rows,
            stable_row_count,
            deferred_row_count,
            unsupported_row_count,
            fail_closed_row_count,
            parallel_api_forbidden_row_count,
            matrix_digest,
        })
    }

    pub fn backend_posture(&self) -> WorthQueryRuntimeBackendPosture {
        self.backend_posture
    }

    pub fn rows(&self) -> &[WorthQueryRuntimePublicSupportMatrixRow] {
        &self.rows
    }

    pub fn stable_row_count(&self) -> usize {
        self.stable_row_count
    }

    pub fn deferred_row_count(&self) -> usize {
        self.deferred_row_count
    }

    pub fn unsupported_row_count(&self) -> usize {
        self.unsupported_row_count
    }

    pub fn fail_closed_row_count(&self) -> usize {
        self.fail_closed_row_count
    }

    pub fn parallel_api_forbidden_row_count(&self) -> usize {
        self.parallel_api_forbidden_row_count
    }

    pub fn matrix_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.matrix_digest
    }

    pub fn row(&self, surface: &str) -> Option<&WorthQueryRuntimePublicSupportMatrixRow> {
        self.rows.iter().find(|row| row.surface() == surface)
    }

    pub fn row_for_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Option<&WorthQueryRuntimePublicSupportMatrixRow> {
        self.rows
            .iter()
            .find(|row| row.facade_family() == Some(family))
    }
}

fn shared_read_pinning_boundary_closure_contract_digest() -> Result<String, IdentityDerivationDenial>
{
    let certification = WorthQuerySharedReadPinningCertification::support_gate_required()?;
    debug_assert_ne!(certification.closure().posture().as_str(), "closed");
    Ok(certification.closure().closure_digest().to_string())
}

fn milestone_nine_seven_derived_closure_contract_digest() -> Result<String, IdentityDerivationDenial>
{
    Ok(
        WorthQueryMilestoneNineSevenDerivedClosure::support_profile_publication_contract()?
            .closure_digest()
            .to_string(),
    )
}
