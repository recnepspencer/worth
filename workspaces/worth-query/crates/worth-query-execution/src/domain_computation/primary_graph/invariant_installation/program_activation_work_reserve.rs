//! What the platform's own branch program activation read costs one installed
//! rule, and how lowering pays for it instead of the rule's author.
//!
//! A program-rostered host reads the branch program activation record before an
//! author's rule body runs, to decide whether that rule speaks for the
//! candidate at all. Relational charges that structural read against the rule's
//! own work meter, so an author declaring exactly the units their body needs
//! would be refused for work they never asked for — and would be refused only
//! on a program-rostered host, which is a difference the author cannot see.
//! Lowering therefore raises the meter by this reserve, so the author still
//! receives exactly the units they declared.

use std::num::NonZeroU64;

use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

/// Work units the platform's branch program activation read spends: one
/// structural entity-aspect state read, charged once per candidate evaluation.
pub(super) const PROGRAM_ACTIVATION_READ_RESERVE: u64 = 1;

/// The work meter one lowered rule actually runs under.
///
/// A rule lowered without program selection performs no platform read, so its
/// meter is exactly what its author declared. A rule lowered with program
/// selection carries the reserve on top, and an author-declared budget too
/// large to carry it is a typed installation denial rather than a silently
/// saturated meter.
pub(super) fn lowered_work_budget(
    declared: NonZeroU64,
    subject: &str,
    reads_program_activation: bool,
) -> Result<NonZeroU64, WorthQueryPrimaryGraphInstallationDenial> {
    if !reads_program_activation {
        return Ok(declared);
    }
    declared
        .checked_add(PROGRAM_ACTIVATION_READ_RESERVE)
        .ok_or_else(|| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::ProgramActivationWorkReserveOverflow,
                subject,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{lowered_work_budget, PROGRAM_ACTIVATION_READ_RESERVE};
    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;
    use std::num::NonZeroU64;

    fn declared(units: u64) -> NonZeroU64 {
        NonZeroU64::new(units).expect("a declared work budget is non-zero")
    }

    #[test]
    fn a_selection_bearing_rule_keeps_every_unit_its_author_declared() {
        assert_eq!(
            lowered_work_budget(declared(1), "bounded-rule", true)
                .expect("the platform reserve fits above a small declared budget"),
            declared(1 + PROGRAM_ACTIVATION_READ_RESERVE)
        );
    }

    #[test]
    fn a_rule_on_a_host_with_no_program_pays_nothing_for_a_read_it_never_performs() {
        assert_eq!(
            lowered_work_budget(declared(64), "bounded-rule", false)
                .expect("a non-program host lowers the declared budget unchanged"),
            declared(64)
        );
    }

    #[test]
    fn a_declared_budget_that_cannot_carry_the_reserve_denies_installation() {
        let denial = lowered_work_budget(declared(u64::MAX), "bounded-rule", true)
            .expect_err("a budget that cannot carry the reserve must not install");
        assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::ProgramActivationWorkReserveOverflow
        );
        assert_eq!(denial.subject(), "bounded-rule");
    }

    #[test]
    fn the_largest_budget_that_still_carries_the_reserve_installs() {
        assert_eq!(
            lowered_work_budget(
                declared(u64::MAX - PROGRAM_ACTIVATION_READ_RESERVE),
                "bounded-rule",
                true
            )
            .expect("the reserve fits exactly at the ceiling"),
            declared(u64::MAX)
        );
    }
}
