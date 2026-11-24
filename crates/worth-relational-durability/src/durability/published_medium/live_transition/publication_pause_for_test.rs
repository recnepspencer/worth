//! Owner-scoped rendezvous while a live transition holds its reservation.
use super::{
    sealed, LiveBranchTransitionStep, PositionedBranchTransition,
    RelationalBranchTransitionDurableStep, RelationalDurableStepStop,
    RelationalTransitionPublication,
};
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation;

pub struct PausedBranchTransitionForTest<'request, 'authority> {
    step: LiveBranchTransitionStep<'request, 'authority>,
    pause: Mutex<Option<PublicationPause>>,
}

struct PublicationPause {
    reached: mpsc::SyncSender<()>,
    resume: mpsc::Receiver<()>,
}

pub struct BranchTransitionPublicationPauseForTest {
    reached: mpsc::Receiver<()>,
    resume: mpsc::SyncSender<()>,
}

impl BranchTransitionPublicationPauseForTest {
    pub fn await_publication(&self) {
        self.reached
            .recv_timeout(Duration::from_secs(30))
            .expect("transition reached its publication step");
    }

    pub fn release(&self) {
        let _ = self.resume.try_send(());
    }
}

impl Drop for BranchTransitionPublicationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}

impl<'request, 'authority> PausedBranchTransitionForTest<'request, 'authority> {
    pub(super) fn arm(
        step: LiveBranchTransitionStep<'request, 'authority>,
    ) -> (Self, BranchTransitionPublicationPauseForTest) {
        let (reached, observed) = mpsc::sync_channel(1);
        let (resume, released) = mpsc::sync_channel(1);
        (
            Self {
                step,
                pause: Mutex::new(Some(PublicationPause {
                    reached,
                    resume: released,
                })),
            },
            BranchTransitionPublicationPauseForTest {
                reached: observed,
                resume,
            },
        )
    }
}

impl sealed::Step for PausedBranchTransitionForTest<'_, '_> {}

impl RelationalBranchTransitionDurableStep for PausedBranchTransitionForTest<'_, '_> {
    type Stop = RelationalDurableStepStop;

    fn caller_association(&self) -> Option<&RelationalCallerAssociation> {
        self.step.caller_association()
    }

    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop> {
        let pause = self.pause.lock().unwrap().take();
        if let Some(pause) = pause {
            pause.reached.send(()).unwrap();
            pause
                .resume
                .recv_timeout(Duration::from_secs(30))
                .expect("transition publication resumes");
        }
        self.step.take(positioned)
    }
}
