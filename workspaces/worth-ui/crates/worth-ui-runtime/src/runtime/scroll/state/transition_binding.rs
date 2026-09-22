//! Binding between the authoritative owner records and their pending
//! transition targets.
//!
//! The offset in an owner record is accepted truth; a transition target is a
//! derived intention that leads it. This file is the only place the two meet:
//! staging reads the owner's accepted offset, bounds and axis policy, and
//! reconciliation pulls a pending target back inside freshly reconciled bounds
//! or retires it when the owner has been reincarnated.

impl super::UiScrollRuntimeState {
    /// Stage or advance one owner's transition target from an admitted coarse
    /// wheel event.
    pub(crate) fn stage_wheel_transition(
        &mut self,
        entry: crate::runtime::scroll::UiScrollChainEntry,
        input: crate::runtime::scroll::transition::UiScrollWheelInput,
    ) -> Result<
        crate::runtime::scroll::transition::UiScrollTransitionTarget,
        crate::runtime::scroll::transition::UiScrollTransitionDenial,
    > {
        let record = *self
            .owners
            .get(&entry.owner())
            .ok_or(crate::runtime::scroll::transition::UiScrollTransitionDenial::UnknownOwner)?;
        if record.incarnation != entry.incarnation() {
            return Err(
                crate::runtime::scroll::transition::UiScrollTransitionDenial::StaleIncarnation,
            );
        }
        self.transition_targets.accumulate_wheel(
            entry.owner(),
            entry.incarnation(),
            input,
            crate::runtime::scroll::transition::UiScrollTransitionBasis::new(
                record.offset,
                record.bounds,
                record.axes,
            ),
        )
    }

    pub(crate) fn transition_target(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    ) -> Option<crate::runtime::scroll::transition::UiScrollTransitionTarget> {
        self.transition_targets.target(owner, incarnation)
    }

    /// Retire one owner's transition: thumb capture taking direct control,
    /// owner removal, modality loss, or an explicit cancellation.
    pub(crate) fn retire_transition(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> bool {
        self.transition_targets.retire(owner)
    }

    /// Retire every pending transition whose owner's accepted offset has
    /// arrived at the target it was settling toward.
    ///
    /// A settle ends when the content is where it was going, which the horizon
    /// only approximates: a track that reaches its endpoint early leaves a
    /// target standing, and the next notch would then accumulate against an
    /// intention the reader can no longer see any distance to.
    pub(crate) fn retire_reached_transitions(&mut self) -> usize {
        let reached = self
            .transition_targets
            .pending_owners()
            .filter(|(owner, target)| {
                self.owners.get(owner).is_some_and(|record| {
                    record.incarnation == target.incarnation()
                        && record.offset == target.target_offset()
                })
            })
            .map(|(owner, _)| owner)
            .collect::<Vec<_>>();
        for owner in &reached {
            self.transition_targets.retire(*owner);
        }
        reached.len()
    }

    /// Every pending settle, paired with a mounted occurrence whose ownership
    /// chain names its owner.
    ///
    /// Succession records which owner is settling; the ownership catalog
    /// records which occurrences that owner belongs to. A settle needs both
    /// halves to be ended -- the owner names the target to retire, the
    /// occurrence is what turns that owner into the Motion target the sampler
    /// and the track are filed under -- and joining them is something only
    /// Scroll can do, because only Scroll holds both lists. Without it a
    /// lifecycle boundary can retire an intention but not the motion carrying
    /// it out.
    ///
    /// A region owner belongs to exactly one occurrence, so it appears once. A
    /// surface or viewport owner belongs to every occurrence presented under
    /// it and appears once per occurrence, which is right: its content motion
    /// is filed per occurrence too.
    pub(crate) fn pending_settle_occurrences(
        &self,
    ) -> Vec<(
        crate::runtime::scroll::UiScrollOwnerIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )> {
        let settling = self
            .transition_targets
            .pending_owners()
            .map(|(owner, _)| owner)
            .collect::<Vec<_>>();
        if settling.is_empty() {
            return Vec::new();
        }
        let mut joined = Vec::with_capacity(settling.len());
        for mounted in self.ownership_instances() {
            let Ok(chain) = self.ownership_chain(mounted) else {
                continue;
            };
            for owner in chain.owners().iter().copied() {
                if settling.contains(&owner) {
                    joined.push((owner, mounted));
                }
            }
        }
        joined
    }

    #[cfg(test)]
    pub(crate) fn pending_transition_count(&self) -> usize {
        self.transition_targets.pending_count()
    }

    /// A pending target may never continue toward an offset the reconciled
    /// bounds no longer admit, so bounds reconciliation re-clamps it in the
    /// same preparation that moved the accepted offset. A reincarnated owner
    /// retires its predecessor's target instead of inheriting it.
    pub(in crate::runtime::scroll) fn reconcile_transition_bounds(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        bounds: crate::runtime::scroll::UiScrollBounds,
    ) -> crate::runtime::scroll::transition::UiScrollTransitionReclampOutcome {
        self.transition_targets
            .reconcile_bounds(owner, incarnation, bounds)
    }

    pub(in crate::runtime::scroll) fn release_transitions(&mut self) -> usize {
        self.transition_targets.clear()
    }
}
