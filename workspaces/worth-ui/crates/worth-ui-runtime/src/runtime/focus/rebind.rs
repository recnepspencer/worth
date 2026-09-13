use std::collections::BTreeMap;

pub(crate) struct UiPreparedFocusMountedReconciliation {
    participation: Option<UiPreparedFocusMountedParticipation>,
    predecessor_revision: u64,
    predecessor_structure: u64,
    predecessor_appearance: super::UiFocusAppearancePosture,
    transition: Option<super::UiFocusPlan>,
    appearance: super::UiFocusAppearancePosture,
    nodes_visited: u32,
    installed: u32,
}

struct UiPreparedFocusMountedParticipation {
    participants: BTreeMap<super::UiFocusScopeIdentity, Vec<super::UiFocusParticipant>>,
    participant_index:
        BTreeMap<super::UiFocusParticipantIdentity, (super::UiFocusScopeIdentity, usize)>,
    nodes_visited: u32,
}

impl super::UiFocusRuntimeState {
    pub(crate) fn prepare_mounted_reconciliation(
        &self,
        snapshot: &crate::mounting::UiMountedFocusParticipationSnapshot,
    ) -> Result<UiPreparedFocusMountedReconciliation, super::UiFocusRoutingDenial> {
        if self
            .pending_portal
            .values()
            .any(|transition| transition.frame() == snapshot.frame())
        {
            let installed = u32::try_from(
                snapshot
                    .participants()
                    .iter()
                    .filter(|participant| {
                        participant.support()
                            != crate::capability::ComponentFocusSupport::NotFocusable
                    })
                    .count(),
            )
            .map_err(|_| super::UiFocusRoutingDenial::VisitCounterOverflow)?;
            return Ok(UiPreparedFocusMountedReconciliation {
                participation: None,
                predecessor_revision: self.revision,
                predecessor_structure: self.structural_revision,
                predecessor_appearance: self.appearance_posture(),
                transition: None,
                appearance: self.appearance_posture(),
                nodes_visited: snapshot.nodes_visited(),
                installed,
            });
        }
        self.structural_revision
            .checked_add(1)
            .ok_or(super::UiFocusRoutingDenial::RevisionExhausted)?;
        let participation = prepare_participation(snapshot, &self.participants)?;
        let nodes_visited = participation.nodes_visited;
        let installed = u32::try_from(participation.participant_index.len())
            .map_err(|_| super::UiFocusRoutingDenial::VisitCounterOverflow)?;
        let transition = self.prepare_reconciliation_transition(&participation);
        let next = transition.as_ref().map_or(self.current, |plan| plan.next());
        let appearance_revision = if next != self.current {
            self.appearance_revision
                .checked_add(1)
                .ok_or(super::UiFocusRoutingDenial::RevisionExhausted)?
        } else {
            self.appearance_revision
        };
        if transition.is_some() && self.revision == u64::MAX {
            return Err(super::UiFocusRoutingDenial::RevisionExhausted);
        }
        Ok(UiPreparedFocusMountedReconciliation {
            predecessor_revision: self.revision,
            predecessor_structure: self.structural_revision,
            predecessor_appearance: self.appearance_posture(),
            appearance: self.appearance_posture_for(next, appearance_revision),
            transition,
            participation: Some(participation),
            nodes_visited,
            installed,
        })
    }

    pub(crate) fn admits_mounted_reconciliation(
        &self,
        prepared: &UiPreparedFocusMountedReconciliation,
    ) -> bool {
        self.revision == prepared.predecessor_revision
            && self.structural_revision == prepared.predecessor_structure
            && self.appearance_posture() == prepared.predecessor_appearance
    }

    pub(crate) fn commit_mounted_reconciliation(
        &mut self,
        prepared: UiPreparedFocusMountedReconciliation,
    ) -> Result<super::UiFocusReconciliationReceipt, super::UiFocusRoutingDenial> {
        if !self.admits_mounted_reconciliation(&prepared) {
            return Err(super::UiFocusRoutingDenial::StalePlan);
        }
        let Some(participation) = prepared.participation else {
            return Ok(super::UiFocusReconciliationReceipt::new(
                None,
                prepared.nodes_visited,
                prepared.installed,
            ));
        };
        self.install_prepared_participation(participation);
        let transition = prepared
            .transition
            .map(|plan| self.commit(plan))
            .transpose()?;
        Ok(super::UiFocusReconciliationReceipt::new(
            transition,
            prepared.nodes_visited,
            prepared.installed,
        ))
    }

    pub(crate) fn reconcile_mounted_participation(
        &mut self,
        snapshot: &crate::mounting::UiMountedFocusParticipationSnapshot,
    ) -> Result<super::UiFocusReconciliationReceipt, super::UiFocusRoutingDenial> {
        let prepared = self.prepare_mounted_reconciliation(snapshot)?;
        self.commit_mounted_reconciliation(prepared)
    }

    pub(super) fn install_mounted_participation(
        &mut self,
        snapshot: &crate::mounting::UiMountedFocusParticipationSnapshot,
    ) -> Result<u32, super::UiFocusRoutingDenial> {
        self.structural_revision
            .checked_add(1)
            .ok_or(super::UiFocusRoutingDenial::RevisionExhausted)?;
        let prepared = prepare_participation(snapshot, &self.participants)?;
        let installed = u32::try_from(prepared.participant_index.len())
            .map_err(|_| super::UiFocusRoutingDenial::VisitCounterOverflow)?;
        self.install_prepared_participation(prepared);
        Ok(installed)
    }

    fn install_prepared_participation(&mut self, prepared: UiPreparedFocusMountedParticipation) {
        self.structural_revision = self
            .structural_revision
            .checked_add(1)
            .expect("prepared participation reserves its structural revision");
        self.participants = prepared.participants;
        self.participant_index = prepared.participant_index;
        if self.active_descendant.is_some_and(|active| {
            self.exact_participant(
                active.scope(),
                active.descendant(),
                active.descendant_incarnation(),
            )
            .is_err()
        }) {
            self.active_descendant = None;
        }
    }

    fn prepare_reconciliation_transition(
        &self,
        participation: &UiPreparedFocusMountedParticipation,
    ) -> Option<super::UiFocusPlan> {
        let current = self.current?;
        let successor = participation
            .participant_index
            .get(&current.participant())
            .filter(|(scope, _)| *scope == current.scope())
            .map(|(scope, index)| participation.participants[scope][*index])
            .filter(|participant| participant.incarnation() == current.incarnation());
        let (next, cause) = match successor {
            Some(successor) if current.exact_participant() == successor => return None,
            Some(successor) => (Some(successor), super::UiFocusCause::RebindPreserved),
            None => (
                participation
                    .participants
                    .get(&current.scope())
                    .and_then(|rows| rows.iter().find(|row| row.container().is_none()))
                    .copied(),
                super::UiFocusCause::RebindFallback,
            ),
        };
        Some(self.plan_for(next, cause, u32::from(next.is_some())))
    }
}

fn prepare_participation(
    snapshot: &crate::mounting::UiMountedFocusParticipationSnapshot,
    previous: &BTreeMap<super::UiFocusScopeIdentity, Vec<super::UiFocusParticipant>>,
) -> Result<UiPreparedFocusMountedParticipation, super::UiFocusRoutingDenial> {
    let mut participants = BTreeMap::<_, Vec<_>>::new();
    let mut nodes_visited = snapshot.nodes_visited();
    for (scope, rows) in previous {
        if snapshot.retains_surface(scope.semantic_surface()) {
            nodes_visited = nodes_visited
                .checked_add(
                    u32::try_from(rows.len())
                        .map_err(|_| super::UiFocusRoutingDenial::VisitCounterOverflow)?,
                )
                .ok_or(super::UiFocusRoutingDenial::VisitCounterOverflow)?;
            participants.insert(*scope, rows.clone());
        }
    }
    for participant in focusable_participants(snapshot) {
        participants
            .entry(participant.scope())
            .or_default()
            .push(participant);
    }
    for scoped in participants.values_mut() {
        scoped.sort_by_key(|participant| participant.mounted_order());
    }
    let mut participant_index = BTreeMap::new();
    for (scope, scoped) in &participants {
        for (index, participant) in scoped.iter().enumerate() {
            participant_index.insert(participant.identity(), (*scope, index));
        }
    }
    u32::try_from(participant_index.len())
        .map_err(|_| super::UiFocusRoutingDenial::VisitCounterOverflow)?;
    Ok(UiPreparedFocusMountedParticipation {
        participants,
        participant_index,
        nodes_visited,
    })
}

pub(super) fn focusable_participants(
    snapshot: &crate::mounting::UiMountedFocusParticipationSnapshot,
) -> Vec<super::UiFocusParticipant> {
    snapshot
        .participants()
        .iter()
        .copied()
        .filter_map(super::UiFocusParticipant::from_mounted)
        .collect()
}

impl UiPreparedFocusMountedReconciliation {
    pub(crate) const fn appearance_posture(&self) -> super::UiFocusAppearancePosture {
        self.appearance
    }
}
