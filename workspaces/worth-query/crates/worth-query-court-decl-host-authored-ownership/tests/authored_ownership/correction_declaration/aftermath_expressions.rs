//! Correction-mechanism and reconciliation expressions are checked at association.
use super::*;
use std::cell::Cell;
thread_local! { static RECONCILIATIONS: Cell<usize> = const { Cell::new(0) }; }
struct EvaluationScope;
impl Drop for EvaluationScope {
    fn drop(&mut self) {
        RECONCILIATIONS.with(|value| value.set(0));
    }
}
pub(super) fn mechanism<const DAMAGE: u8>() -> DeclaredCorrectionMechanism<S> {
    let postcondition = DeclaredAftermathPostcondition::BusinessPostcondition {
        identity: "reconciled".into(),
    };
    match DAMAGE {
        21 => DeclaredCorrectionMechanism::compensation_binding::<Correction<DAMAGE>>(
            DeclaredAftermathPostcondition::ExactPriorTruth,
        ),
        22 => {
            DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new("", postcondition))
        }
        29 => DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
            "compensate",
            DeclaredAftermathPostcondition::InvariantRestored {
                invariant: "".into(),
            },
        )),
        30 => {
            DeclaredCorrectionMechanism::compensation_binding::<Correction<DAMAGE>>(postcondition)
        }
        23 => DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
            "compensate",
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "".into(),
            },
        )),
        _ => DeclaredCorrectionMechanism::RecordedInverse(DeclaredRecordedInverse::new(
            if DAMAGE == 24 { "" } else { "RetireOp" },
            DeclaredLoweringCorrespondenceRef::new(if DAMAGE == 25 { "" } else { "restore" }),
            if DAMAGE == 26 {
                postcondition
            } else {
                DeclaredAftermathPostcondition::ExactPriorTruth
            },
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(RawId::<false>::at("Id"))],
                16,
            ),
        )),
    }
}
fn invalid_mechanism<const DAMAGE: u8>(cause: DeclaredCorrectionMechanismDenial) {
    let _scope = EvaluationScope;
    assert_eq!(
        compose::<DAMAGE>().unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidCorrectionMechanism {
            correction: "declaration.correction.v1".into(),
            cause
        }
    );
}
#[test]
fn a_compensation_binding_cannot_claim_exact_prior_truth() {
    invalid_mechanism::<21>(DeclaredCorrectionMechanismDenial::CompensationRejectsExactPriorTruth);
}
#[test]
fn an_empty_compensating_operation_is_refused_at_composition() {
    invalid_mechanism::<22>(DeclaredCorrectionMechanismDenial::EmptyCompensatingOperation);
}
#[test]
fn an_empty_compensation_postcondition_is_refused_at_composition() {
    invalid_mechanism::<23>(DeclaredCorrectionMechanismDenial::EmptyCompensationPostcondition);
}
#[test]
fn an_empty_inverse_operation_is_refused_at_composition() {
    invalid_mechanism::<24>(DeclaredCorrectionMechanismDenial::EmptyInverseOperation);
}
#[test]
fn an_empty_lowering_correspondence_is_refused_at_composition() {
    invalid_mechanism::<25>(DeclaredCorrectionMechanismDenial::EmptyLoweringCorrespondence);
}
#[test]
fn a_recorded_inverse_requires_exact_prior_truth_at_composition() {
    invalid_mechanism::<26>(
        DeclaredCorrectionMechanismDenial::RecordedInverseRequiresExactPriorTruth,
    );
}
#[test]
fn an_empty_reconciliation_procedure_is_refused_at_composition() {
    invalid_mechanism::<28>(DeclaredCorrectionMechanismDenial::EmptyReconciliationProcedure);
}
#[test]
fn a_lawful_recorded_inverse_expression_composes() {
    assert!(compose::<27>().is_ok());
}

#[test]
fn an_empty_invariant_postcondition_is_refused_at_composition() {
    invalid_mechanism::<29>(DeclaredCorrectionMechanismDenial::EmptyCompensationPostcondition);
}
#[test]
fn a_lawful_compensation_expression_composes() {
    assert!(compose::<30>().is_ok());
}
#[test]
fn a_lawful_reconciliation_expression_composes_once() {
    let _scope = EvaluationScope;
    assert!(compose::<31>().is_ok());
    assert_eq!(RECONCILIATIONS.with(Cell::get), 1);
}

pub(super) fn reconciliation<const DAMAGE: u8>() -> Option<DeclaredReconciliationProcedure> {
    matches!(DAMAGE, 28 | 31).then(|| {
        RECONCILIATIONS.with(|value| value.set(value.get() + 1));
        DeclaredReconciliationProcedure::new(if DAMAGE == 28 { "" } else { "reconcile" })
    })
}
