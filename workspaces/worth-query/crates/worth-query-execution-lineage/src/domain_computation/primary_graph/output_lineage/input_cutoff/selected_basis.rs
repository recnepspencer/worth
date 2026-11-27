//! One admitted Native position paired with its exact issued snapshot handle.

use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use super::verification::InputCutoffVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::CertificationBasis;

/// This is preparation evidence for cutoff verification, not mutation authority.
/// Its private fields cannot pair an unrelated position with the issued handle.
pub struct PreparedInputCutoffBasis<'snapshot> {
    basis: CertificationBasis<'snapshot>,
    positioned: PositionedRelationalSnapshot,
}

impl<'snapshot> PreparedInputCutoffBasis<'snapshot> {
    pub fn into_positioned(self) -> PositionedRelationalSnapshot {
        self.positioned
    }
    pub fn prepare(
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'snapshot>,
    ) -> Result<Self, InputCutoffVerificationStop> {
        let positioned = runtime
            .read_truth()
            .positioned_snapshot(basis.snapshot)
            .map_err(|_| InputCutoffVerificationStop::SelectedSourceUnavailable)?;
        Ok(Self {
            basis,
            positioned,
        })
    }

    pub fn in_runtime(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<(CertificationBasis<'snapshot>, &PositionedRelationalSnapshot), InputCutoffVerificationStop>
    {
        if runtime.runtime_instance_id() != self.positioned.runtime_instance_id() {
            return Err(InputCutoffVerificationStop::SelectedSourceMismatch);
        }
        Ok((self.basis, &self.positioned))
    }
}

mod court_doors_for_test;
