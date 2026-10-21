use super::super::*;

mod preparation;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_replaced_interest(
        &self,
        replaced: &WorthQueryOutputDemandInterest,
        replacement: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if replaced.key != replacement.key {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retired = match preparation::prepare(&mut state, &replaced.key, &replacement.key)? {
                Some(mut prepared) => {
                    let state = &mut *prepared.mutation;
                    let mut old_record = state.records.get_mut(&replaced.key).unwrap();
                    let mut old_slots = std::mem::take(&mut old_record.performed_obligations);
                    let old_bytes =
                        old_slots.capacity() * std::mem::size_of::<PerformedOutputObligation>();
                    old_record.finish();
                    let mut next_record = state.records.get_mut(&replacement.key).unwrap();
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
                        growth.install(&mut next_record);
                    }
                    next_record
                        .source_commits
                        .append(&mut prepared.added_commits);
                    next_record.performed_obligations = prepared.slots;
                    // Retired Vec backings still exist after this guard is released.
                    // Their credits remain reserved until their physical Drop.
                    next_record.finish();
                    state.install_required_member(prepared.required_member);
                    let (_, claims) = state.finish_superseded_row(&replaced.key, subject);
                    let member = prepared
                        .mutation
                        .remove_required_member_if_released_detached(&replaced.key);
                    (
                        old_slots,
                        next_slots,
                        prepared.added_commits,
                        old_bytes + next_bytes,
                        claims,
                        member,
                    )
                }
                _ => {
                    let priced = &mut state;
                    let (_, claims) = priced.finish_superseded_row(&replaced.key, subject);
                    let member = priced.remove_required_member_if_released_detached(&replaced.key);
                    (Vec::new(), Vec::new(), Vec::new(), 0, claims, member)
                }
            };
            drop(state);
            drop(retired);
        }
        Ok(())
    }
}
