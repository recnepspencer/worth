//! The wave cannot advance a successor without consuming its paid turn.
use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

type Refusal = fn(CompanionPreflightStop) -> WorthQueryOutputDemandDenial;

pub(in super::super) struct WaveCheckpoint<'a, Schema: ApplicationSchema> {
    progress: &'a mut RequiredFreshProgress<Schema>,
    admission: &'a mut InvalidationEditAdmission,
    refusal: Refusal,
}
pub(in super::super) enum WaveCheckpointTurn<'a, Schema: ApplicationSchema> {
    Current(SelectedReadyReadmission),
    Awaiting(PaidCheckpointStage<'a, Schema>),
}
pub(in super::super) struct PaidCheckpointStage<'a, Schema: ApplicationSchema> {
    progress: &'a mut RequiredFreshProgress<Schema>,
    work: CheckpointWork<'a>,
}

impl<Schema: ApplicationSchema + 'static> RequiredFreshProgress<Schema> {
    fn advance_checkpoint(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        paid: PaidCheckpointAdvance<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.successor
            .advance_checkpoint(phase, runtime, request, paid)
    }

    pub(in super::super) fn checkpoint<'a>(
        &'a mut self,
        admission: &'a mut InvalidationEditAdmission,
        refusal: Refusal,
    ) -> WaveCheckpoint<'a, Schema> {
        WaveCheckpoint {
            progress: self,
            admission,
            refusal,
        }
    }
}
impl<'a, Schema: ApplicationSchema + 'static> WaveCheckpoint<'a, Schema> {
    pub(in super::super) fn inspect(
        self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<WaveCheckpointTurn<'a, Schema>, WorthQueryOutputDemandDenial> {
        match inspect_checkpoint(self.admission, self.refusal, |admission| {
            registry.interest_ready_readmission(self.progress.interest(), admission)
        })? {
            CheckpointInspection::Current(ready) => Ok(WaveCheckpointTurn::Current(ready)),
            CheckpointInspection::Awaiting(work) => {
                Ok(WaveCheckpointTurn::Awaiting(PaidCheckpointStage {
                    progress: self.progress,
                    work,
                }))
            }
        }
    }
}
impl<Schema: ApplicationSchema + 'static> PaidCheckpointStage<'_, Schema> {
    pub(in super::super) fn advance(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.work.advance(|paid| {
            self.progress
                .advance_checkpoint(phase, runtime, request, paid)
        })
    }
}

// This value owns the original ledger borrow; a stage cannot substitute
// another admission or advance after a refused turn.
enum CheckpointInspection<'a, T> {
    Current(T),
    Awaiting(CheckpointWork<'a>),
}
fn inspect_checkpoint<'a, T>(
    admission: &'a mut InvalidationEditAdmission,
    refusal: Refusal,
    inspect: impl FnOnce(
        &mut InvalidationEditAdmission,
    ) -> Result<Option<T>, WorthQueryOutputDemandDenial>,
) -> Result<CheckpointInspection<'a, T>, WorthQueryOutputDemandDenial> {
    let work = CheckpointWork::admit(admission, refusal)?;
    Ok(match inspect(work.admission)? {
        Some(current) => CheckpointInspection::Current(current),
        None => CheckpointInspection::Awaiting(work),
    })
}
/// The raw successor consumes this grant; only the paid stage can issue it.
pub(super) struct PaidCheckpointAdvance<'a> {
    admission: &'a mut InvalidationEditAdmission,
}
impl<'a> PaidCheckpointAdvance<'a> {
    pub(super) fn into_admission(self) -> &'a mut InvalidationEditAdmission {
        self.admission
    }
}

struct CheckpointWork<'a> {
    admission: &'a mut InvalidationEditAdmission,
    refusal: Refusal,
}
impl<'a> CheckpointWork<'a> {
    fn admit(
        admission: &'a mut InvalidationEditAdmission,
        refusal: Refusal,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        admission.charge_external_work(2).map_err(refusal)?;
        Ok(Self { admission, refusal })
    }
    fn advance(
        self,
        advance: impl FnOnce(PaidCheckpointAdvance<'_>) -> Result<bool, WorthQueryOutputDemandDenial>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.admission
            .charge_external_work(1)
            .map_err(self.refusal)?;
        advance(PaidCheckpointAdvance {
            admission: self.admission,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::mvcc::CompanionPreflightBudget;
    fn admission(work: u64) -> InvalidationEditAdmission {
        InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: work,
            maximum_preparation_bytes: 0,
        })
    }
    fn refusal(_: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
        work_denial()
    }
    #[test]
    fn two_checkpoint_stages_and_the_current_turn_pay_the_declared_turn_work() {
        let inspections = 3;
        let stages = 2;
        let expected = inspections * 2 + stages;
        let mut admission = admission(expected);
        for turn in 0..inspections {
            match inspect_checkpoint(&mut admission, refusal, |_| {
                Ok((turn == stages).then_some(()))
            })
            .unwrap()
            {
                CheckpointInspection::Current(()) => assert_eq!(turn, stages),
                CheckpointInspection::Awaiting(paid) => {
                    assert!(paid.advance(|_| Ok(true)).unwrap())
                }
            }
        }
        assert_eq!(
            admission.charged_work(),
            8,
            "three inspections cost six; two real advances cost two"
        );
        assert_eq!(admission.charged_work(), expected);
    }
    #[test]
    fn a_turn_cannot_inspect_or_advance_before_its_work_is_admitted() {
        let mut empty = admission(0);
        let mut inspected = false;
        assert!(inspect_checkpoint(&mut empty, refusal, |_| {
            inspected = true;
            Ok(Some(()))
        })
        .is_err());
        assert!(!inspected, "an unfunded turn performs no readiness lookup");
        let mut two = admission(2);
        let CheckpointInspection::Awaiting(paid) =
            inspect_checkpoint::<()>(&mut two, refusal, |_| Ok(None)).unwrap()
        else {
            panic!("the declared checkpoint is not Current");
        };
        let mut advanced = false;
        assert!(paid
            .advance(|_| {
                advanced = true;
                Ok(true)
            })
            .is_err());
        assert!(!advanced, "an unfunded stage performs no checkpoint work");
        assert_eq!(two.charged_work(), 2);
    }
    #[test]
    fn an_unfunded_checkpoint_turn_never_inspects_or_advances_the_successor() {
        let required_turns = 3;
        let paid_turns = required_turns - 1;
        let mut admission = admission(paid_turns * (2 + 1));
        let mut inspections = 0;
        let mut completed = 0;
        let refusal = super::super::super::progression::admission_denial;
        let stop = loop {
            let paid = match inspect_checkpoint::<()>(&mut admission, refusal, |_| {
                inspections += 1;
                Ok(None)
            }) {
                Ok(CheckpointInspection::Awaiting(paid)) => paid,
                Ok(CheckpointInspection::Current(())) => panic!("all three stages were declared"),
                Err(stop) => break stop,
            };
            paid.advance(|_| {
                completed += 1;
                Ok(true)
            })
            .unwrap();
        };
        assert_eq!(stop.kind(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded);
        assert_eq!(completed, paid_turns);
        assert_eq!(
            inspections, paid_turns,
            "the unfunded third turn performs no lookup"
        );
        assert_eq!(admission.charged_work(), paid_turns * (2 + 1));
    }
}
