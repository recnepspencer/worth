use super::super::*;

mod preparation;

fn replacement_key_work(
    replaced: &WorthQueryOutputDemandKey,
    replacement: &WorthQueryOutputDemandKey,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    super::super::key_comparison::comparison_work([replaced, replacement])
        .and_then(|work| work.total.checked_add(8))
        .ok_or_else(replacement_work_denial)
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_replaced_interest(
        &self,
        replaced: &WorthQueryOutputDemandInterest,
        replacement: &WorthQueryOutputDemandInterest,
        subject: &str,
        request_admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let key_work = replacement_key_work(&replaced.key, &replacement.key)?;
        request_admission
            .charge_external_work(u64::try_from(key_work).map_err(|_| replacement_work_denial())?)
            .map_err(|_| replacement_work_denial())?;
        if replaced.key != replacement.key {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retired = if let Some(mut prepared) = preparation::prepare(
                &mut state,
                &replaced.key,
                &replacement.key,
                request_admission,
            )? {
                let state = prepared.mutation.state_mut();
                let old_record = state.records.get_mut(&replaced.key).unwrap();
                let mut old_slots = std::mem::take(&mut old_record.performed_obligations);
                let old_bytes =
                    old_slots.capacity() * std::mem::size_of::<PerformedOutputObligation>();
                let next_record = state.records.get_mut(&replacement.key).unwrap();
                let mut next_slots = std::mem::take(&mut next_record.performed_obligations);
                let next_bytes =
                    next_slots.capacity() * std::mem::size_of::<PerformedOutputObligation>();
                prepared.slots.append(&mut next_slots);
                prepared
                    .slots
                    .extend(old_slots.extract_if(.., |obligation| {
                        obligation
                            .source
                            .same_semantic_source(&replacement.key.source)
                    }));
                if let Some(growth) = prepared.commit_growth {
                    growth.install(next_record);
                }
                next_record
                    .source_commits
                    .append(&mut prepared.added_commits);
                next_record.performed_obligations = prepared.slots;
                // Retired Vec backings still exist after this guard is released.
                // Their credits remain reserved until their physical Drop.
                state.obligation_reserved_bytes += prepared.bytes;
                state.install_required_member(prepared.required_member);
                let member = prepared.mutation.release_member(&replaced.key);
                let refund = member.refund_required_bytes;
                drop(member);
                let state = prepared.mutation.state_mut();
                state.required_reserved_bytes =
                    state.required_reserved_bytes.saturating_sub(refund);
                Some((
                    old_slots,
                    next_slots,
                    prepared.added_commits,
                    old_bytes + next_bytes,
                ))
            } else {
                None
            };
            drop(state);
            let retired_bytes = retired.as_ref().map_or(0, |retired| retired.3);
            drop(retired);
            if retired_bytes != 0 {
                // This scalar owner cleanup is prepaid before transfer. A new
                // admission cannot borrow the credit before the backing dies.
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.obligation_reserved_bytes = state
                    .obligation_reserved_bytes
                    .checked_sub(retired_bytes)
                    .expect("retired transfer credits remain reserved");
            }
            self.finish_superseded(replaced, subject);
        }
        Ok(())
    }
}

fn replacement_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "replacement required-member work exceeds the carried request",
    )
}

#[cfg(test)]
mod comparison_tests {
    #[test]
    fn replacement_compares_whole_declared_keys() {
        let (short, long) = super::super::super::key_comparison::fixtures::keys();
        assert_eq!(
            super::replacement_key_work(&short, &long).unwrap(),
            (4 + 1 + 1 + 10) + (4 + 15 + 16 + 10) + 8
        );
    }
}
