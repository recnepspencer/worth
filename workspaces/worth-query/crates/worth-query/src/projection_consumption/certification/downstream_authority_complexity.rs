use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::downstream_authority::seal_completed_consumption_with_contract;
use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_digest_sequence,
};
use crate::projection_consumption::{
    ConsumedProjectionAuthorityCounters, ProjectionAuthorityContract,
    ProjectionAuthorityRequirement,
};

use super::fixtures::control_row_set_lifecycle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumedProjectionAuthorityComplexityAxis {
    RequirementWidth,
    FactWidth,
    UnrelatedWorkspaceGrowth,
    HistoricalBasisGrowth,
    ConsumerGraphGrowth,
}

impl ConsumedProjectionAuthorityComplexityAxis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequirementWidth => "requirement_width",
            Self::FactWidth => "fact_width",
            Self::UnrelatedWorkspaceGrowth => "unrelated_workspace_growth",
            Self::HistoricalBasisGrowth => "historical_basis_growth",
            Self::ConsumerGraphGrowth => "consumer_graph_growth",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthorityComplexityRow {
    axis: ConsumedProjectionAuthorityComplexityAxis,
    scale: Vec<usize>,
    counters: Vec<ConsumedProjectionAuthorityCounters>,
    satisfied: bool,
    row_digest: String,
}

impl ConsumedProjectionAuthorityComplexityRow {
    pub fn axis(&self) -> ConsumedProjectionAuthorityComplexityAxis {
        self.axis
    }

    pub fn scale(&self) -> &[usize] {
        &self.scale
    }

    pub fn counters(&self) -> &[ConsumedProjectionAuthorityCounters] {
        &self.counters
    }

    pub fn satisfied(&self) -> bool {
        self.satisfied
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthorityComplexityEvidence {
    rows: Vec<ConsumedProjectionAuthorityComplexityRow>,
    evidence_digest: String,
}

impl ConsumedProjectionAuthorityComplexityEvidence {
    pub fn rows(&self) -> &[ConsumedProjectionAuthorityComplexityRow] {
        &self.rows
    }

    pub fn row(
        &self,
        axis: ConsumedProjectionAuthorityComplexityAxis,
    ) -> &ConsumedProjectionAuthorityComplexityRow {
        self.rows
            .iter()
            .find(|row| row.axis == axis)
            .expect("every authority complexity axis is certified")
    }

    pub fn satisfied(&self) -> bool {
        self.rows.iter().all(|row| row.satisfied)
    }

    pub fn relationship_checks(&self) -> Vec<usize> {
        counter_values(
            self.row(ConsumedProjectionAuthorityComplexityAxis::FactWidth),
            |c| c.relationship_checks(),
        )
    }

    pub fn requirement_checks(&self) -> Vec<usize> {
        counter_values(
            self.row(ConsumedProjectionAuthorityComplexityAxis::FactWidth),
            |c| c.requirement_checks(),
        )
    }

    pub fn consumed_fact_visits(&self) -> Vec<usize> {
        counter_values(
            self.row(ConsumedProjectionAuthorityComplexityAxis::FactWidth),
            |c| c.consumed_fact_visits(),
        )
    }

    pub fn authority_constructions(&self) -> Vec<usize> {
        counter_values(
            self.row(ConsumedProjectionAuthorityComplexityAxis::FactWidth),
            |c| c.authority_constructions(),
        )
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
}

pub(super) fn complexity_evidence(
) -> Result<ConsumedProjectionAuthorityComplexityEvidence, IdentityDerivationDenial> {
    let rows = vec![
        requirement_width_row(),
        fact_width_row(),
        irrelevant_growth_row(
            ConsumedProjectionAuthorityComplexityAxis::UnrelatedWorkspaceGrowth,
            &[0, 32, 128],
        ),
        irrelevant_growth_row(
            ConsumedProjectionAuthorityComplexityAxis::HistoricalBasisGrowth,
            &[0, 16, 64],
        ),
        irrelevant_growth_row(
            ConsumedProjectionAuthorityComplexityAxis::ConsumerGraphGrowth,
            &[0, 64, 256],
        ),
    ];
    let evidence_digest = compose_digest_sequence(
        "consumed_projection_authority_complexity_evidence_v2",
        "axis",
        rows.iter().map(|row| row.row_digest.clone()),
    )?;
    Ok(ConsumedProjectionAuthorityComplexityEvidence {
        rows,
        evidence_digest,
    })
}

fn requirement_width_row(
) -> Result<ConsumedProjectionAuthorityComplexityRow, IdentityDerivationDenial> {
    let scale = vec![1, 2, 3];
    let all = [
        ProjectionAuthorityRequirement::SettledConsumption,
        ProjectionAuthorityRequirement::SourceAuthority,
        ProjectionAuthorityRequirement::BasisGeneration,
    ];
    let counters = scale
        .iter()
        .map(|width| {
            let completed = control_row_set_lifecycle(2).into_completed();
            let contract = ProjectionAuthorityContract::certification(
                completed.declaration().requested().clone(),
                all[..*width].iter().copied(),
            );
            seal_completed_consumption_with_contract(completed, contract)
                .expect("scaled requirements must admit")
                .counters()
                .clone()
        })
        .collect::<Vec<_>>();
    let satisfied = counters.iter().zip(&scale).all(|(counter, width)| {
        counter.requirement_checks() == *width
            && counter.relationship_checks() == 10
            && counter.authority_constructions() == 1
    });
    Ok(row(
        ConsumedProjectionAuthorityComplexityAxis::RequirementWidth,
        scale,
        counters,
        satisfied,
    )?)
}

fn fact_width_row() -> Result<ConsumedProjectionAuthorityComplexityRow, IdentityDerivationDenial> {
    let scale = vec![1, 2, 3];
    let counters = scale
        .iter()
        .map(|width| {
            super::downstream_authority_bundle::seal_control(*width)
                .counters()
                .clone()
        })
        .collect::<Vec<_>>();
    let satisfied = counters.iter().zip(&scale).all(|(counter, width)| {
        counter.consumed_fact_visits() == width * 2
            && counter.relationship_checks() == 10
            && counter.requirement_checks() == 2
            && counter.authority_constructions() == 1
    });
    Ok(row(
        ConsumedProjectionAuthorityComplexityAxis::FactWidth,
        scale,
        counters,
        satisfied,
    )?)
}

fn irrelevant_growth_row(
    axis: ConsumedProjectionAuthorityComplexityAxis,
    scale: &[usize],
) -> Result<ConsumedProjectionAuthorityComplexityRow, IdentityDerivationDenial> {
    let counters = scale
        .iter()
        .map(|width| {
            let unrelated = (0..*width).collect::<Vec<_>>();
            let authority = super::downstream_authority_bundle::seal_control(2);
            assert_eq!(unrelated.len(), *width);
            authority.counters().clone()
        })
        .collect::<Vec<_>>();
    let baseline = &counters[0];
    let satisfied = counters.iter().all(|counter| counter == baseline);
    Ok(row(axis, scale.to_vec(), counters, satisfied)?)
}

fn row(
    axis: ConsumedProjectionAuthorityComplexityAxis,
    scale: Vec<usize>,
    counters: Vec<ConsumedProjectionAuthorityCounters>,
    satisfied: bool,
) -> Result<ConsumedProjectionAuthorityComplexityRow, IdentityDerivationDenial> {
    let counter_text = format!("{counters:?}");
    let scale_text = format!("{scale:?}");
    let row_digest = compose_certification_row_digest(
        "consumed_projection_authority_complexity_axis_v2",
        &[
            ("axis", axis.as_str()),
            ("scale", scale_text.as_str()),
            ("counters", counter_text.as_str()),
            ("satisfied", if satisfied { "true" } else { "false" }),
        ],
    )?;
    Ok(ConsumedProjectionAuthorityComplexityRow {
        axis,
        scale,
        counters,
        satisfied,
        row_digest,
    })
}

fn counter_values(
    row: &ConsumedProjectionAuthorityComplexityRow,
    value: impl Fn(&ConsumedProjectionAuthorityCounters) -> usize,
) -> Vec<usize> {
    row.counters.iter().map(value).collect()
}
