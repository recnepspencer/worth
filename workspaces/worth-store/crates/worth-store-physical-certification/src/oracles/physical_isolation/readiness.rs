use crate::{CheckpointInterlockObservation, CompactionInterlockObservation, OracleFamilyKind};

use super::super::{
    OracleDenial, OracleVerdictBasis, PhysicalOracleNonClaim, PhysicalProofOracle,
    PhysicalProofOracleKind, PhysicalProofOracleVerdict,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoMixedRootOracle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OldReaderSeesOldRootOracle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostSwapReaderSeesNewRootOracle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockedReclaimUntilReleaseOracle;

impl PhysicalProofOracle for NoMixedRootOracle {
    fn oracle_kind(&self) -> PhysicalProofOracleKind {
        PhysicalProofOracleKind::NoMixedRoot
    }

    fn family_kind(&self) -> OracleFamilyKind {
        OracleFamilyKind::PhysicalIsolationReadinessShape
    }

    fn judge_basis(
        &self,
        basis: OracleVerdictBasis,
    ) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
        require_satisfied_read_interlock_fact(
            self.oracle_kind(),
            basis,
            |observation| observation.no_mixed_root(),
            |observation| observation.no_mixed_root(),
        )
    }
}

impl PhysicalProofOracle for OldReaderSeesOldRootOracle {
    fn oracle_kind(&self) -> PhysicalProofOracleKind {
        PhysicalProofOracleKind::OldReaderSeesOldRoot
    }

    fn family_kind(&self) -> OracleFamilyKind {
        OracleFamilyKind::PhysicalIsolationReadinessShape
    }

    fn judge_basis(
        &self,
        basis: OracleVerdictBasis,
    ) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
        require_satisfied_read_interlock_fact(
            self.oracle_kind(),
            basis,
            |observation| observation.old_reachability_deferred(),
            |observation| observation.old_reader_retained_old_root(),
        )
    }
}

impl PhysicalProofOracle for PostSwapReaderSeesNewRootOracle {
    fn oracle_kind(&self) -> PhysicalProofOracleKind {
        PhysicalProofOracleKind::PostSwapReaderSeesNewRoot
    }

    fn family_kind(&self) -> OracleFamilyKind {
        OracleFamilyKind::PhysicalIsolationReadinessShape
    }

    fn judge_basis(
        &self,
        basis: OracleVerdictBasis,
    ) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
        require_satisfied_read_interlock_fact(
            self.oracle_kind(),
            basis,
            |observation| observation.post_cutover_plan_matches_publication(),
            |observation| observation.post_publication_reader_observed_new_epoch(),
        )
    }
}

impl PhysicalProofOracle for BlockedReclaimUntilReleaseOracle {
    fn oracle_kind(&self) -> PhysicalProofOracleKind {
        PhysicalProofOracleKind::BlockedReclaimUntilRelease
    }

    fn family_kind(&self) -> OracleFamilyKind {
        OracleFamilyKind::PhysicalIsolationReadinessShape
    }

    fn judge_basis(
        &self,
        basis: OracleVerdictBasis,
    ) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
        require_satisfied_compaction_fact(self.oracle_kind(), basis, |observation| {
            observation.blocked_reclaim_until_release()
        })
    }
}

fn require_satisfied_read_interlock_fact(
    oracle: PhysicalProofOracleKind,
    basis: OracleVerdictBasis,
    compaction_predicate: impl FnOnce(CompactionInterlockObservation) -> bool,
    checkpoint_predicate: impl FnOnce(CheckpointInterlockObservation) -> bool,
) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
    // This family judges local readiness shape only. Its mandatory non-claim
    // excludes live reader visibility, byte integrity, and actual retention.
    if let Some(observation) = basis.checkpoint_interlock() {
        if !checkpoint_predicate(observation) {
            return Err(OracleDenial::CheckpointInterlockObservationDenied { oracle });
        }
        return Ok(PhysicalProofOracleVerdict::satisfied(
            OracleFamilyKind::PhysicalIsolationReadinessShape,
            oracle,
            basis,
            [PhysicalOracleNonClaim::PhysicalIsolationCorrectness],
        ));
    }
    require_satisfied_compaction_fact(oracle, basis, compaction_predicate)
}

fn require_satisfied_compaction_fact(
    oracle: PhysicalProofOracleKind,
    basis: OracleVerdictBasis,
    predicate: impl FnOnce(CompactionInterlockObservation) -> bool,
) -> Result<PhysicalProofOracleVerdict, OracleDenial> {
    let Some(observation) = basis.compaction_interlock() else {
        return Err(OracleDenial::MissingCompactionInterlockObservation);
    };
    if !predicate(observation) {
        return Err(OracleDenial::CompactionInterlockObservationDenied { oracle });
    }
    Ok(PhysicalProofOracleVerdict::satisfied(
        OracleFamilyKind::PhysicalIsolationReadinessShape,
        oracle,
        basis,
        [PhysicalOracleNonClaim::PhysicalIsolationCorrectness],
    ))
}
