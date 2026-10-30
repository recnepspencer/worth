//! Each advance retains the ordinary operation's native posture until completion or close.
use super::*;

#[derive(Debug)]
pub enum StageRefusal {
    Authorization(PreparationRefusal),
    Construction(retention_action::ConstructionFailure),
    Advance(WorthQuerySourceOnlyDenial),
    Close(WorthQueryOperationCloseDenial),
}
#[derive(Debug)]
pub enum StageEvent {
    Progress,
    StepCompleted,
    AllStaged,
    Terminal,
    Refused(StageRefusal),
}
impl Staging<'_> {
    pub fn advance(&mut self, model: &Model<'_>, caller: &Caller<'_>) -> StageEvent {
        if let Err(cause) = preparation::authorize(model, caller, &self.proposal) {
            return StageEvent::Refused(StageRefusal::Authorization(cause));
        }
        if self.next == self.proposal.intents.len() {
            return StageEvent::AllStaged;
        }
        if self.step.is_none() {
            let AuthoredIntent::Retention(input) = &self.proposal.intents[self.next];
            // prepare() checked representability of every ordinal before forking.
            let key = retention::StepKey {
                attempt: self.metadata.attempt,
                ordinal: u64::try_from(self.next).expect("prepared ordinal"),
            };
            match retention_action::construct(self.host, self.branch(), caller, input, key) {
                Ok(operation) => self.step = Some(Step::Pending(operation)),
                Err(cause) => return StageEvent::Refused(StageRefusal::Construction(cause)),
            }
        }
        let request = self
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch());
        let step = self.step.take().expect("constructed ordinary step");
        let completed = match step {
            Step::Pending(operation) => match operation.advance(&request) {
                WorthQuerySourceOnlyAdvance::Progress(operation) => {
                    self.step = Some(Step::Pending(operation));
                    return StageEvent::Progress;
                }
                WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                    self.step = Some(Step::Pending(operation));
                    return StageEvent::Refused(StageRefusal::Advance(denial));
                }
                WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                    self.step = Some(Step::Terminal(terminal));
                    return StageEvent::Terminal;
                }
                WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
            },
            Step::Completed(completed) => completed,
            Step::Terminal(terminal) => {
                self.step = Some(Step::Terminal(terminal));
                return StageEvent::Terminal;
            }
        };
        match completed.close_with_evidence(&request) {
            Ok((_closed, evidence)) => {
                // Capacity was reserved before creation; no allocation follows this effect.
                self.completed.push(evidence);
                self.next += 1;
                StageEvent::StepCompleted
            }
            Err((completed, denial)) => {
                self.step = Some(Step::Completed(completed));
                StageEvent::Refused(StageRefusal::Close(denial))
            }
        }
    }
}
