//! Quotes for framework-owned invariant evaluation and evidence allocations.
use super::super::packets::InvariantWorkPacket;
use crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration as Registration;
use crate::validation::engine::budget::InvariantBudget;

pub(super) fn prepare_registration(
    packet: &InvariantWorkPacket<'_>,
    budget: Option<&dyn InvariantBudget>,
) -> bool {
    let Some(budget) = budget else {
        return true;
    };
    if !budget.claim_result(std::mem::size_of::<crate::validation::data::InvariantVerdict>() as u64)
    {
        return false;
    }
    match &packet.registration {
        Registration::Native(registration) => budget.claim_result(
            super::super::envelope::native_rule_bytes(&registration.rule),
        ),
        Registration::Custom { registration, .. }
        | Registration::CustomNotApplicable { registration, .. } => {
            let access = registration.access_contract();
            let kinds = access.read_entity_kinds.len()
                + access.read_relation_kinds.len()
                + access.affected_entity_kinds.len()
                + access.affected_relation_kinds.len();
            let context =
                std::mem::size_of::<crate::validation::data::CustomInvariantAccessContract>()
                    + 2 * std::mem::size_of::<usize>()
                    + kinds * std::mem::size_of::<crate::identity::data::KindId>();
            let proposal =
                super::super::envelope::proposal_bytes(packet.observation.proposal_identity());
            let meter = if matches!(&packet.registration, Registration::Custom { .. }) {
                crate::validation::custom_rule::CustomPreparationBudget::allocation_bytes()
                    + crate::validation::custom_rule::CustomPreparationBudget::evaluation_meter_allocation_bytes()
            } else {
                0
            };
            budget.claim_scratch(context as u64 + proposal + meter) && budget.claim_result(proposal)
        }
    }
}

pub(super) fn format_detail(
    budget: Option<&dyn InvariantBudget>,
    arguments: std::fmt::Arguments<'_>,
) -> Option<String> {
    struct Length(usize);
    impl std::fmt::Write for Length {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut length = Length(0);
    std::fmt::write(&mut length, arguments).ok()?;
    if budget.is_some_and(|budget| !budget.claim_result(length.0 as u64)) {
        return None;
    }
    let mut text = String::with_capacity(length.0);
    std::fmt::write(&mut text, arguments).ok()?;
    Some(text)
}
