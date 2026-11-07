//! Retention expressions are evaluated once and checked by correction composition.
use super::*;
use std::cell::Cell;
thread_local! {
    static EVALUATIONS: Cell<usize> = const { Cell::new(0) };
}
struct EvaluationScope;
impl Drop for EvaluationScope {
    fn drop(&mut self) {
        EVALUATIONS.with(|value| value.set(0));
    }
}

pub(super) fn mechanism<const DAMAGE: u8>() -> DeclaredCorrectionMechanism<S> {
    EVALUATIONS.with(|value| value.set(value.get() + 1));
    let field = || {
        DeclaredPreImageLocus::from_field(RawId::<false>::at(if DAMAGE == 12 { "" } else { "Id" }))
    };
    let relation = || DeclaredPreImageRelationLocus::from_relation(Peer::reference());
    let demand = match DAMAGE {
        34 => {
            let DeclaredCorrectionMechanism::RecordedInverse(created) =
                DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Correction<DAMAGE>>(
                )
            else {
                unreachable!()
            };
            created
                .preimage_demand()
                .clone()
                .with_relations([relation()])
        }
        32 => DeclaredPreImageDemand::relations_only([relation()], 16),
        33 => DeclaredPreImageDemand::new([], 16).with_relations([relation()]),
        10 => DeclaredPreImageDemand::new([], 16),
        11 => DeclaredPreImageDemand::new([field()], 0),
        12 => DeclaredPreImageDemand::new([field()], 16),
        13 => DeclaredPreImageDemand::new([field(), field()], 16),
        14 => DeclaredPreImageDemand::relations_only([relation(), relation()], 16),
        15 => DeclaredPreImageDemand::relations_only([relation()], 0),
        16 => DeclaredPreImageDemand::relations_only([], 16),
        17 => DeclaredPreImageDemand::relations_only(
            [
                DeclaredPreImageRelationLocus::from_relation(ApplicationRelationRef::<
                    S,
                    Peer,
                    NativeNode<false>,
                    NativeNode<false>,
                >::from_schema_identifiers(
                    "",
                    "Node",
                    "Node",
                    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
                )),
            ],
            16,
        ),
        18 => DeclaredPreImageDemand::new([field()], 16).with_relations([relation(), relation()]),
        19 => DeclaredPreImageDemand::new([field()], 0).with_relations([relation()]),
        _ => DeclaredPreImageDemand::new([field()], 16).with_relations([relation()]),
    };
    DeclaredCorrectionMechanism::recorded_inverse_binding::<Correction<DAMAGE>>(demand)
}
fn refused<const DAMAGE: u8>(cause: DeclaredPreImageDemandDenial) {
    let _scope = EvaluationScope;
    assert_eq!(
        compose::<DAMAGE>().unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidCorrectionDemand {
            correction: "declaration.correction.v1".into(),
            cause
        }
    );
    assert_eq!(EVALUATIONS.with(Cell::get), 1);
}
#[test]
fn an_empty_field_demand_is_refused_at_composition() {
    refused::<10>(DeclaredPreImageDemandDenial::EmptyDemand);
}
#[test]
fn a_zero_field_retention_bound_is_refused_at_composition() {
    refused::<11>(DeclaredPreImageDemandDenial::ZeroByteBound);
}
#[test]
fn an_empty_field_axis_is_refused_at_composition() {
    refused::<12>(DeclaredPreImageDemandDenial::EmptyLocusAxis);
}
#[test]
fn duplicate_field_loci_are_refused_at_composition() {
    refused::<13>(DeclaredPreImageDemandDenial::DuplicateLocus);
}
#[test]
fn duplicate_relation_loci_are_refused_at_composition() {
    refused::<14>(DeclaredPreImageDemandDenial::DuplicateLocus);
}
#[test]
fn a_zero_relation_retention_bound_is_refused_at_composition() {
    refused::<15>(DeclaredPreImageDemandDenial::ZeroByteBound);
}
#[test]
fn an_empty_relation_demand_is_refused_at_composition() {
    refused::<16>(DeclaredPreImageDemandDenial::EmptyDemand);
}
#[test]
fn an_empty_relation_axis_is_refused_at_composition() {
    refused::<17>(DeclaredPreImageDemandDenial::EmptyLocusAxis);
}
#[test]
fn duplicate_appended_relations_are_refused_at_composition() {
    refused::<18>(DeclaredPreImageDemandDenial::DuplicateLocus);
}
#[test]
fn appending_relations_does_not_hide_a_zero_field_bound() {
    refused::<19>(DeclaredPreImageDemandDenial::ZeroByteBound);
}
#[test]
fn a_lawful_mixed_demand_composes_from_one_expression() {
    let _scope = EvaluationScope;
    let result = compose::<20>();
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(EVALUATIONS.with(Cell::get), 1);
}

#[test]
fn a_lawful_relation_only_demand_composes() {
    assert!(compose::<32>().is_ok());
}
#[test]
fn composition_checks_the_complete_demand_after_relations_are_added() {
    assert!(compose::<33>().is_ok());
}

#[test]
fn adding_capture_to_a_created_entity_demand_requires_a_byte_bound() {
    refused::<34>(DeclaredPreImageDemandDenial::ZeroByteBound);
}
