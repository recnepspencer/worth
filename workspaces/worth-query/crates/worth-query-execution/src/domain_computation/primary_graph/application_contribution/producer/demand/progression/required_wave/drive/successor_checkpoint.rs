//! Each successor checkpoint turn funds inspection before advancing its stage.
use super::*;

pub(super) enum CheckpointTurn<T> {
    Current(T),
    Awaiting(CheckpointStage),
}
pub(super) struct CheckpointStage(());

pub(super) fn inspect<T>(
    admission: &mut InvalidationEditAdmission,
    lookup: impl FnOnce(
        &mut InvalidationEditAdmission,
    ) -> Result<Option<T>, WorthQueryOutputDemandDenial>,
) -> Result<CheckpointTurn<T>, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(2)
        .map_err(admission_denial)?;
    Ok(match lookup(admission)? {
        Some(ready) => CheckpointTurn::Current(ready),
        None => CheckpointTurn::Awaiting(CheckpointStage(())),
    })
}
impl CheckpointStage {
    pub(super) fn advance(
        self,
        admission: &mut InvalidationEditAdmission,
        advance: impl FnOnce(
            &mut InvalidationEditAdmission,
        ) -> Result<bool, WorthQueryOutputDemandDenial>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(1)
            .map_err(admission_denial)?;
        advance(admission)
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
    #[test]
    fn two_checkpoint_stages_and_the_current_turn_pay_the_declared_turn_work() {
        let inspections = 3;
        let stages = 2;
        let expected = inspections * 2 + stages;
        let mut admission = admission(expected);
        for turn in 0..inspections {
            match inspect(&mut admission, |_| Ok((turn == stages).then_some(()))).unwrap() {
                CheckpointTurn::Current(()) => assert_eq!(turn, stages),
                CheckpointTurn::Awaiting(stage) => {
                    assert!(stage.advance(&mut admission, |_| Ok(true)).unwrap());
                }
            }
        }
        assert_eq!(
            admission.charged_work(),
            expected,
            "three inspected turns and two real stage advances spend 2 + 1 per stage, then 2"
        );
    }
    #[test]
    fn a_turn_cannot_inspect_or_advance_before_its_work_is_admitted() {
        let mut empty = admission(0);
        let mut inspected = false;
        assert!(inspect(&mut empty, |_| {
            inspected = true;
            Ok(Some(()))
        })
        .is_err());
        assert!(!inspected, "an unfunded turn performs no readiness lookup");
        let mut two = admission(2);
        let CheckpointTurn::Awaiting(stage) = inspect::<()>(&mut two, |_| Ok(None)).unwrap() else {
            panic!("the declared checkpoint is not Current");
        };
        let mut advanced = false;
        assert!(stage
            .advance(&mut two, |_| {
                advanced = true;
                Ok(true)
            })
            .is_err());
        assert!(!advanced, "an unfunded stage performs no checkpoint work");
        assert_eq!(two.charged_work(), 2);
    }
}
