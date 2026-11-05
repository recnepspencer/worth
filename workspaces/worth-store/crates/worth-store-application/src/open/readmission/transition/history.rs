//! The original protected Home cut admits intents, their chain and reservations.
use super::{intent, ApplicationHomeTransitionIntentDenial as Denial, HomeProgramTransitionIntent};
use crate::checkpoint::SelectedHomeSource;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{ApplicationHomeBinding, HomeIdentity};
use worth_store::physical_runtime::*;

pub(crate) struct ReadmittedHomeProgramTransitions {
    intents: ExecutionArray<ReadmittedHomeProgramTransition>,
    attempt_frontier: Option<u64>,
}
struct ReadmittedHomeProgramTransition {
    intent: HomeProgramTransitionIntent,
    _entry: OrderedJournalEntry,
}
impl ReadmittedHomeProgramTransitions {
    pub(crate) fn read(
        serving: &ServingPhysicalRuntime,
        source: &SelectedHomeSource,
        home: HomeIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let name = OrderedJournalName::new("home.transitions").expect("closed Home journal");
        let head = source
            .journal_head(serving, name, request)
            .map_err(Denial::ReadSource)?;
        let count = usize::try_from(head).map_err(|_| Denial::SizeLimit)?;
        let mut admitted =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(Denial::Backing)?;
        let expected_owner =
            crate::world_section::UnclaimedWorldSlot::owner_identity_for_home(home);
        let mut owner = None;
        let mut last_assignment = None;
        for position in 1..=head {
            request.consult().map_err(Denial::Execution)?;
            let (entry, bytes) = source
                .journal_payload(serving, name, position, request)
                .map_err(Denial::ReadSource)?;
            if entry.position() != position || entry.payload_bytes() != bytes.len() as u64 {
                return Err(Denial::SourceMismatch);
            }
            let intent = intent::decode(&bytes, home, position, request)?;
            if admitted.elements().last().is_some_and(
                |previous: &ReadmittedHomeProgramTransition| {
                    previous.intent.successor() != intent.predecessor()
                },
            ) {
                return Err(Denial::BindingLineage);
            }
            let current_owner = intent.plan().predecessor().branch().owner();
            if current_owner != expected_owner || owner.is_some_and(|owner| owner != current_owner)
            {
                return Err(Denial::ForeignWorld);
            }
            owner = Some(current_owner);
            for assignment in intent.assignments() {
                if last_assignment
                    .is_some_and(|previous| assignment.attempt().ordinal() <= previous)
                {
                    return Err(Denial::ReservationOverlap);
                }
                last_assignment = Some(assignment.attempt().ordinal());
            }
            admitted
                .push(ReadmittedHomeProgramTransition {
                    intent,
                    _entry: entry,
                })
                .map_err(Denial::Backing)?;
        }
        // A selected cut is not permission to ignore an unresolved successor.
        if let Some(next) = head.checked_add(1) {
            let journal = serving.ordered_journal(name).map_err(Denial::Journal)?;
            match journal.resolve(next, [0; 32]).map_err(Denial::Read)? {
                OrderedJournalResolution::NotCommitted { head: actual } if actual == head => {}
                OrderedJournalResolution::Unresolved => return Err(Denial::JournalUnresolved),
                OrderedJournalResolution::NotCommitted { .. }
                | OrderedJournalResolution::Committed(_)
                | OrderedJournalResolution::Conflict(_) => {
                    return Err(Denial::JournalSourceChanged)
                }
            }
        }
        Ok(Self {
            intents: admitted.seal().map_err(Denial::Backing)?,
            attempt_frontier: last_assignment
                .map(|last| last.checked_add(1).ok_or(Denial::ReservationExhausted))
                .transpose()?,
        })
    }

    pub(crate) fn pending(
        &self,
        binding: &ApplicationHomeBinding,
    ) -> Result<Option<&HomeProgramTransitionIntent>, Denial> {
        let Some(last) = self.intents.elements().last() else {
            return Ok(None);
        };
        if last.intent.successor() == binding {
            return Ok(None);
        }
        if last.intent.predecessor() == binding {
            return Ok(Some(&last.intent));
        }
        Err(Denial::BindingLineage)
    }
    pub(crate) fn require_successor(
        &self,
        binding: &ApplicationHomeBinding,
        successor: &ApplicationHomeBinding,
    ) -> Result<(), Denial> {
        if let Some(pending) = self.pending(binding)? {
            if pending.successor() != successor {
                return Err(Denial::TransitionPending);
            }
        }
        Ok(())
    }
    pub(crate) fn reserved_attempt_frontier(&self) -> Option<u64> {
        self.attempt_frontier
    }

    pub(crate) fn require_recorded_successor(
        &self,
        recorded: &[u8],
        requested: &ApplicationHomeBinding,
    ) -> Result<(), Denial> {
        let Some(last) = self.intents.elements().last() else {
            return Ok(());
        };
        if crate::descriptor::binding::describes(recorded, last.intent.successor())
            .map_err(Denial::Binding)?
        {
            return Ok(());
        }
        if !crate::descriptor::binding::describes(recorded, last.intent.predecessor())
            .map_err(Denial::Binding)?
        {
            return Err(Denial::BindingLineage);
        }
        if requested != last.intent.successor() {
            return Err(Denial::TransitionPending);
        }
        Ok(())
    }
}

pub(crate) enum HomeTransitionAdmission {
    Current {
        history: ReadmittedHomeProgramTransitions,
    },
    Requested {
        history: ReadmittedHomeProgramTransitions,
        successor: ApplicationHomeBinding,
    },
}

impl HomeTransitionAdmission {
    pub(crate) fn require_resumed_plan(
        &self,
        current: &ApplicationHomeBinding,
        digest: worth_foundational::facade::HomeProgramTransitionPlanDigest,
        steps: u32,
        target: worth_foundational::facade::HomeProgramTransitionTargetMeaning,
    ) -> Result<&HomeProgramTransitionIntent, Denial> {
        match self {
            Self::Current { history: _ } => Err(Denial::TransitionNotRequested),
            Self::Requested { history, successor } => {
                let intent = history
                    .pending(current)?
                    .ok_or(Denial::IntentNotCommitted)?;
                intent.require_plan(successor, digest, steps, target)?;
                Ok(intent)
            }
        }
    }

    pub(crate) fn history(&self) -> &ReadmittedHomeProgramTransitions {
        match self {
            Self::Current { history }
            | Self::Requested {
                history,
                successor: _,
            } => history,
        }
    }
    pub(crate) fn require_fresh_plan(
        &self,
        current: &ApplicationHomeBinding,
    ) -> Result<(), Denial> {
        match self {
            Self::Current { history: _ } => Err(Denial::TransitionNotRequested),
            Self::Requested {
                history,
                successor: _,
            } => {
                if history.pending(current)?.is_some() {
                    return Err(Denial::TransitionResumeRequired);
                }
                Ok(())
            }
        }
    }
    pub(crate) fn require_fresh_successor(
        &self,
        current: &ApplicationHomeBinding,
        proposed: &ApplicationHomeBinding,
    ) -> Result<(), Denial> {
        match self {
            Self::Current { history: _ } => Err(Denial::TransitionNotRequested),
            Self::Requested { history, successor } => {
                if successor != proposed {
                    return Err(Denial::TransitionPending);
                }
                if history.pending(current)?.is_some() {
                    return Err(Denial::TransitionResumeRequired);
                }
                Ok(())
            }
        }
    }
}
