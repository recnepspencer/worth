//! Compare the decision and retained witness under the same admission.
use super::{
    admission, evaluation::RelationalAuthorizationEvaluation,
    RelationalAuthorizationBudgetedObservationStop as Stop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationFreshness,
};

pub(super) fn compare<A: RelationalAuthorizationObservationAdmission>(
    expected: &RelationalAuthorizationObservationEvidence,
    current: RelationalAuthorizationEvaluation,
    admission: &mut A,
) -> Result<RelationalAuthorizationObservationFreshness, Stop<A::Stop>> {
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
