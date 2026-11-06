use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::downstream_authority::{
    seal_completed_consumption, seal_completed_consumption_with_contract,
};
use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_digest_sequence,
};
use crate::projection_consumption::{
    ConsumedProjectionAuthorityCounters, ConsumedProjectionAuthorityDenial,
    ProjectionAuthorityContract, ProjectionAuthorityRequirement,
};

use super::downstream_authority_complexity::{
    complexity_evidence, ConsumedProjectionAuthorityComplexityEvidence,
};
use super::downstream_authority_support::{
    consumed_projection_authority_support_matrix, ConsumedProjectionAuthoritySupportMatrix,
};
use super::fixtures::control_row_set_lifecycle;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ConsumedProjectionAuthorityCertificationLane {
    CanonicalAdmission,
    DeterministicReplay,
    TypedDenial,
    AuthorityProductSupport,
    ExactComplexity,
}

impl ConsumedProjectionAuthorityCertificationLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalAdmission => "canonical_admission",
            Self::DeterministicReplay => "deterministic_replay",
            Self::TypedDenial => "typed_denial",
            Self::AuthorityProductSupport => "authority_product_support",
            Self::ExactComplexity => "exact_complexity",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthorityCertificationRow {
    lane: ConsumedProjectionAuthorityCertificationLane,
    satisfied: bool,
    evidence_detail: String,
    row_digest: String,
}

impl ConsumedProjectionAuthorityCertificationRow {
    pub fn lane(&self) -> ConsumedProjectionAuthorityCertificationLane {
        self.lane
    }

    pub fn satisfied(&self) -> bool {
        self.satisfied
    }

    pub fn evidence_detail(&self) -> &str {
        &self.evidence_detail
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthorityCertificationBundle {
    support_matrix: ConsumedProjectionAuthoritySupportMatrix,
    admitted_counters: ConsumedProjectionAuthorityCounters,
    denial: ConsumedProjectionAuthorityDenial,
    complexity: ConsumedProjectionAuthorityComplexityEvidence,
    rows: Vec<ConsumedProjectionAuthorityCertificationRow>,
    bundle_digest: String,
}

impl ConsumedProjectionAuthorityCertificationBundle {
    pub fn support_matrix(&self) -> &ConsumedProjectionAuthoritySupportMatrix {
        &self.support_matrix
    }

    pub fn admitted_counters(&self) -> &ConsumedProjectionAuthorityCounters {
        &self.admitted_counters
    }

    pub fn denial(&self) -> &ConsumedProjectionAuthorityDenial {
        &self.denial
    }

    pub fn complexity(&self) -> &ConsumedProjectionAuthorityComplexityEvidence {
        &self.complexity
    }

    pub fn rows(&self) -> &[ConsumedProjectionAuthorityCertificationRow] {
        &self.rows
    }

    pub fn bundle_digest(&self) -> &str {
        &self.bundle_digest
    }

    pub fn satisfied(&self) -> bool {
        self.rows.iter().all(|row| row.satisfied)
    }
}

pub fn certify_consumed_projection_authority(
) -> Result<ConsumedProjectionAuthorityCertificationBundle, IdentityDerivationDenial> {
    let admitted = seal_completed_consumption(control_row_set_lifecycle(2)?.into_completed())
        .expect("control authority must seal");
    let replay = seal_completed_consumption(control_row_set_lifecycle(2)?.into_completed())
        .expect("replayed authority must seal");
    let denied_completed = control_row_set_lifecycle(2)?.into_completed();
    let contradictory_contract = ProjectionAuthorityContract::certification(
        denied_completed.declaration().requested().clone(),
        [ProjectionAuthorityRequirement::TargetIdentity],
    );
    let denial = seal_completed_consumption_with_contract(denied_completed, contradictory_contract)
        .expect_err("missing target identity must deny without authority");
    let support_matrix = consumed_projection_authority_support_matrix()?;
    let complexity = complexity_evidence()?;
    let rows = vec![
        row(
            ConsumedProjectionAuthorityCertificationLane::CanonicalAdmission,
            admitted.counters().authority_constructions() == 1,
            format!(
                "authority={}",
                admitted.evidence().source_identity_projection()
            ),
        ),
        row(
            ConsumedProjectionAuthorityCertificationLane::DeterministicReplay,
            admitted.structurally_equivalent(&replay),
            format!("receipt={}", admitted.receipt().receipt_digest()),
        ),
        row(
            ConsumedProjectionAuthorityCertificationLane::TypedDenial,
            denial.counters().authority_constructions() == 0,
            format!("denial={:?}", denial.kind()),
        ),
        row(
            ConsumedProjectionAuthorityCertificationLane::AuthorityProductSupport,
            support_matrix.rows().len()
                == super::ProjectionConsumptionCertifiedSourceSurface::all().len(),
            format!("matrix={}", support_matrix.matrix_digest()),
        ),
        row(
            ConsumedProjectionAuthorityCertificationLane::ExactComplexity,
            complexity.satisfied(),
            format!("slopes={}", complexity.evidence_digest()),
        ),
    ];
    let bundle_digest = compose_digest_sequence(
        "consumed_projection_authority_certification_bundle_v1",
        "row",
        rows.iter().map(|row| row.row_digest.clone()),
    )?;
    Ok(ConsumedProjectionAuthorityCertificationBundle {
        support_matrix,
        admitted_counters: admitted.counters().clone(),
        denial,
        complexity,
        rows,
        bundle_digest,
    })
}

pub(super) fn seal_control(
    row_count: usize,
) -> Result<
    crate::projection_consumption::WorthQueryConsumedProjectionAuthority,
    IdentityDerivationDenial,
> {
    Ok(
        seal_completed_consumption(control_row_set_lifecycle(row_count)?.into_completed())
            .expect("scaled control authority must seal"),
    )
}

fn row(
    lane: ConsumedProjectionAuthorityCertificationLane,
    satisfied: bool,
    evidence_detail: String,
) -> Result<ConsumedProjectionAuthorityCertificationRow, IdentityDerivationDenial> {
    let row_digest = compose_certification_row_digest(
        "consumed_projection_authority_certification_row_v1",
        &[
            ("lane", lane.as_str()),
            ("satisfied", if satisfied { "true" } else { "false" }),
            ("evidence", evidence_detail.as_str()),
        ],
    )?;
    Ok(ConsumedProjectionAuthorityCertificationRow {
        lane,
        satisfied,
        evidence_detail,
        row_digest,
    })
}
