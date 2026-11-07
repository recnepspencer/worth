use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::counters::EffectLifecycleCounters;
use super::super::inventory::{EffectLoweredArtifactKind, EffectReceiptArtifactKind};
use super::super::planning::EffectAuthorityOwner;
use super::super::support_contract::{
    deferred_support_contract, support_deferred_neighbors, support_denial_kinds,
    EffectDeferredNeighborFamily, EffectDeferredSupportContract,
};
use super::super::support_matrix_rows::support_rows;
use super::super::taxonomy::{DeniedEffectEligibilityKind, EffectFamily};
use super::lookup::support_decision_for;
use super::row::EffectLifecycleSupportRow;
use super::{
    EffectSupportCause, EffectSupportDecision, EffectSupportPosture,
    EFFECT_LIFECYCLE_IDENTITY_SCOPE,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleSupportMatrix {
    rows: Vec<EffectLifecycleSupportRow>,
    matrix_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecycleSupportMatrix {
    pub fn rows(&self) -> &[EffectLifecycleSupportRow] {
        &self.rows
    }

    pub fn matrix_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.matrix_identity
    }

    pub fn matrix_for_reporting(&self) -> &str {
        self.matrix_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleSupportDiscovery {
    requested_basis_family: BasisFamily,
    requested_effect_family: EffectFamily,
    posture: EffectSupportPosture,
    cause: EffectSupportCause,
    matched_row: Option<EffectLifecycleSupportRow>,
    matched_row_identity: Option<WorthQueryEvidenceIdentity>,
    support_matrix_identity: WorthQueryEvidenceIdentity,
    discovery_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl EffectLifecycleSupportDiscovery {
    fn new(
        requested_basis_family: BasisFamily,
        requested_effect_family: EffectFamily,
        decision: EffectSupportDecision,
        support_matrix_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let EffectSupportDecision {
            posture,
            cause,
            matched_row,
            rows_consulted,
        } = decision;
        let matched_row_identity = matched_row.as_ref().map(|row| row.row_identity().clone());
        let counters = EffectLifecycleCounters::support_lookup(rows_consulted);
        let mut discovery = WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_lifecycle_support_discovery_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_family"),
                requested_basis_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("effect_family"),
                requested_effect_family.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
            .field_shape(WorthQueryEvidenceTag::new("cause"), cause.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("matrix"),
                &support_matrix_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("counters"),
                &counters.evidence_identity(),
            );
        discovery = match matched_row_identity.as_ref() {
            Some(row_identity) => discovery
                .field_evidence_identity(WorthQueryEvidenceTag::new("matched_row"), row_identity),
            None => discovery.field_shape(WorthQueryEvidenceTag::new("matched_row"), "unsupported"),
        };
        let discovery_identity = discovery.seal()?;
        Ok(Self {
            requested_basis_family,
            requested_effect_family,
            posture,
            cause,
            matched_row,
            matched_row_identity,
            support_matrix_identity,
            discovery_identity,
            counters,
        })
    }

    pub fn requested_basis_family(&self) -> BasisFamily {
        self.requested_basis_family
    }

    pub fn requested_effect_family(&self) -> EffectFamily {
        self.requested_effect_family
    }

    pub fn posture(&self) -> EffectSupportPosture {
        self.posture
    }

    pub fn cause(&self) -> EffectSupportCause {
        self.cause
    }

    pub fn authority_owner(&self) -> Option<EffectAuthorityOwner> {
        self.matched_row.as_ref().map(|row| row.authority_owner())
    }

    pub fn requires_rebind(&self) -> bool {
        self.posture == EffectSupportPosture::RebindRequired
    }

    pub fn supported_lowering(&self) -> Option<EffectLoweredArtifactKind> {
        self.matched_row
            .as_ref()
            .map(|row| row.lowered_artifact_kind())
    }

    pub fn receipt_family(&self) -> Option<EffectReceiptArtifactKind> {
        self.matched_row
            .as_ref()
            .map(|row| row.receipt_artifact_kind())
    }

    pub fn denial_kinds(&self) -> &'static [DeniedEffectEligibilityKind] {
        support_denial_kinds(self.posture, self.cause)
    }

    pub fn deferred_neighbors(&self) -> &'static [EffectDeferredNeighborFamily] {
        support_deferred_neighbors(self.requested_effect_family)
    }

    pub fn deferred_contract(
        &self,
    ) -> Result<Option<EffectDeferredSupportContract>, IdentityDerivationDenial> {
        Ok(deferred_support_contract(self.cause)?)
    }

    pub fn matched_row_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.matched_row_identity.as_ref()
    }

    pub fn matched_row_for_reporting(&self) -> Option<&str> {
        self.matched_row_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn support_matrix_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_matrix_identity
    }

    pub fn support_matrix_for_reporting(&self) -> &str {
        self.support_matrix_identity.as_str()
    }

    pub fn discovery_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.discovery_identity
    }

    pub fn discovery_for_reporting(&self) -> &str {
        self.discovery_identity.as_str()
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

pub fn effect_lifecycle_support_matrix(
) -> Result<EffectLifecycleSupportMatrix, IdentityDerivationDenial> {
    let rows = support_rows()
        .iter()
        .map(|row| {
            EffectLifecycleSupportRow::new(
                row.basis_family,
                row.effect_family,
                row.authority_owner,
                row.lowered_artifact_kind,
                row.receipt_artifact_kind,
                row.posture,
                row.cause,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let row_identities = rows
        .iter()
        .map(|row| row.row_identity().clone())
        .collect::<Vec<_>>();
    let matrix_identity = WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_lifecycle_support_matrix_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
        .seal()?;
    Ok(EffectLifecycleSupportMatrix {
        rows,
        matrix_identity,
    })
}

pub fn discover_effect_lifecycle_support(
    basis_family: BasisFamily,
    effect_family: EffectFamily,
) -> Result<EffectLifecycleSupportDiscovery, IdentityDerivationDenial> {
    let support_matrix = effect_lifecycle_support_matrix()?;
    Ok(EffectLifecycleSupportDiscovery::new(
        basis_family,
        effect_family,
        support_decision_for(basis_family, effect_family)?,
        support_matrix.matrix_identity().clone(),
    )?)
}
