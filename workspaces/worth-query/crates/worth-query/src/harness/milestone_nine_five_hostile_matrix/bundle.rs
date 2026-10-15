use crate::harness::certification::CertificationMatrix;

use super::axes::MilestoneNineFivePerturbationClass;
use super::builders;
use super::row::{
    MilestoneNineFiveHostileLaneBundle, MilestoneNineFiveHostileMatrix,
    MilestoneNineFiveHostileRejectionBundle,
};

pub struct MilestoneNineFiveHostileMatrixAdapter;

impl MilestoneNineFiveHostileMatrixAdapter {
    pub fn debt_close_hostile_certification_matrix_test() -> MilestoneNineFiveHostileMatrix {
        CertificationMatrix::<
            MilestoneNineFivePerturbationClass,
            MilestoneNineFiveHostileLaneBundle,
            MilestoneNineFiveHostileRejectionBundle,
        > {
            suite_name: "Milestone 9.5 Debt-Close Hostile Certification Matrix",
            rows: builders::canonical_rows(),
            rejection_rows: builders::rejection_rows(),
        }
    }
}
