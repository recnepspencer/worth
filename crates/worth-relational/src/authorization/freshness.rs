use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

use super::{
    admission::{self, UnrestrictedObservation},
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationFreshness, RelationalAuthorizationPlanAdmissionStop,
};

impl RelationalRuntime {
    pub fn compare_authorization_observation_budgeted<
        A: RelationalAuthorizationObservationAdmission,
    >(
        &self,
        expected: &RelationalAuthorizationObservationEvidence,
        snapshot: &SnapshotHandle,
        admission: &mut A,
    ) -> Result<RelationalAuthorizationObservationFreshness, Stop<A::Stop>> {
        let plan = match expected
            .comparison_plan_admitted(snapshot, |work, bytes| admission.prepare(work, bytes))
        {
            Ok(plan) => plan,
            Err(RelationalAuthorizationPlanAdmissionStop::Plan(_)) => {
                return Ok(RelationalAuthorizationObservationFreshness::Stale);
            }
            Err(RelationalAuthorizationPlanAdmissionStop::Admission(stop)) => {
                return Err(Stop::Admission(stop));
            }
            Err(RelationalAuthorizationPlanAdmissionStop::AccountingOverflow) => {
                return Err(Stop::AccountingOverflow);
            }
        };
        let current = match self.evaluate_authorization_admitted(&plan, admission) {
            Ok(current) => current,
            Err(Stop::Native(_) | Stop::ExactBasisRequired) => {
                return Ok(RelationalAuthorizationObservationFreshness::Stale);
            }
            Err(Stop::Admission(stop)) => return Err(Stop::Admission(stop)),
            Err(Stop::AccountingOverflow) => return Err(Stop::AccountingOverflow),
        };
        admission::prepare(admission, 2, 0)?;
        if current.paths.len() != expected.paths().len() {
            return Ok(RelationalAuthorizationObservationFreshness::Stale);
        }
        for (current, expected) in current.paths.iter().zip(expected.paths()) {
            // Equality checks the decision and optional witness. Each entity
            // witness compares partition, slot and generation coordinates.
            admission::prepare(admission, 3, 0)?;
            let witnesses = (current.witness(), expected.witness());
            if let (Some(left), Some(right)) = witnesses {
                admission::prepare(admission, 2, 0)?;
                let left_len = left.entities().len();
                let right_len = right.entities().len();
                if left_len == right_len {
                    let work = left_len.checked_mul(3).ok_or(Stop::AccountingOverflow)?;
                    admission::prepare(
                        admission,
                        u64::try_from(work).map_err(|_| Stop::AccountingOverflow)?,
                        0,
                    )?;
                }
            }
            admission::prepare(admission, 3, 0)?;
            if matches!(witnesses, (Some(_), Some(_))) {
                admission::prepare(admission, 2, 0)?;
            }
            if !current.has_same_decision_and_witness(expected) {
                return Ok(RelationalAuthorizationObservationFreshness::Stale);
            }
        }
        Ok(RelationalAuthorizationObservationFreshness::Fresh)
    }

    pub fn compare_authorization_observation(
        &self,
        expected: &RelationalAuthorizationObservationEvidence,
        snapshot: SnapshotHandle,
    ) -> RelationalAuthorizationObservationFreshness {
        match self.compare_authorization_observation_budgeted(
            expected,
            &snapshot,
            &mut UnrestrictedObservation,
        ) {
            Ok(freshness) => freshness,
            Err(Stop::AccountingOverflow) => RelationalAuthorizationObservationFreshness::Stale,
            Err(Stop::Admission(impossible)) => match impossible {},
            Err(Stop::Native(_) | Stop::ExactBasisRequired) => {
                RelationalAuthorizationObservationFreshness::Stale
            }
        }
    }
}
