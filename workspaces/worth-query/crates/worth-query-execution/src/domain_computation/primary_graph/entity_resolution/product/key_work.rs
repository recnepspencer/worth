use super::{
    entity_denial, InvalidationEditAdmission, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

/// Fund the installed lookup's key comparisons and three candidate visits.
pub(super) fn admit(
    admission: &InvalidationEditAdmission,
    key_bytes: u64,
    field: &str,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    let exhausted = || {
        entity_denial(
            WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
            field,
        )
    };
    let maximum_key_work = key_bytes.checked_mul(4).ok_or_else(exhausted)?;
    if admission
        .remaining_work_amount()
        .is_some_and(|remaining| remaining < maximum_key_work.saturating_add(3))
    {
        return Err(exhausted());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::mvcc::CompanionPreflightBudget;

    #[test]
    fn key_work_above_word_max_is_admitted_by_the_u64_amount() {
        let maximum = 1_u64 << 40;
        let key_bytes = 1_u64 << 30;
        // Four comparisons plus three visits cost 4294967299, above u32::MAX
        // and below the exact remainder 1099511627759 after the prior debit.
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(maximum),
            maximum_preparation_bytes: None,
        });
        admission.charge_external_work(17).unwrap();
        assert!(
            admit(&admission, key_bytes, "identity").is_ok(),
            "entity key work compares the u64 amount, never the count bound"
        );
        assert_eq!(
            admission.remaining_work_amount(),
            Some(1_099_511_627_759),
            "the key-work comparison retains the exact u64 remainder"
        );
        assert_eq!(
            admission.charged_work(),
            17,
            "preflight comparison does not spend the key work"
        );
    }
}
