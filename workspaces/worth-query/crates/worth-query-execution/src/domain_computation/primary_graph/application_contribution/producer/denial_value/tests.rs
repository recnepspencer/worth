//! The declared binding is the only codec of a producer's domain denial value.
use super::{
    WorthQueryProducerDenialDeclaration as Declaration, WorthQueryProducerDenialDecodeDenial,
    WorthQueryProducerDenialProjection, WorthQueryProducerDenialValue,
    WorthQueryProjectedProducerDenial, WorthQueryUndeclaredProducerDenial,
};
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::application_schema::ValueDecodeBounds;

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
struct PlanRefusal {
    rule: u64,
    evidence: String,
}
worth_query_declaration::worth_query_value_binding!(RefusalBinding for PlanRefusal {
    identity: "tests.plan-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(96, 64, 4, 4),
});
worth_query_declaration::worth_query_value_binding!(OtherBinding for PlanRefusal {
    identity: "tests.other-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(96, 64, 4, 4),
});
worth_query_declaration::worth_query_value_binding!(UnboundedBinding for String {
    identity: "tests.unbounded-refusal.v1", scalar: String,
    validate: validate_unbounded, encode: encode_unbounded, decode: decode_unbounded,
});

fn validate_unbounded(_: &String) -> Result<(), &'static str> {
    Ok(())
}
fn encode_unbounded(value: &str) -> InternedString {
    value.to_owned().into()
}
fn decode_unbounded(value: InternedString) -> Option<String> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    Some(value)
}

fn refusal(evidence: &str) -> PlanRefusal {
    PlanRefusal {
        rule: 7,
        evidence: evidence.to_owned(),
    }
}

#[test]
fn declared_value_round_trips_through_its_binding_without_loss() {
    let original = refusal("edge 4 crosses edge 9");
    let carried = <RefusalBinding as Declaration<PlanRefusal>>::declared_value(&original);
    assert_eq!(carried.value_identity(), Some("tests.plan-refusal.v1"));
    let bytes = carried
        .bytes()
        .expect("a declared value carries its canonical bytes");
    assert!(!bytes.is_empty());
    assert_eq!(carried.decode::<RefusalBinding>(), Ok(original));
}

#[test]
fn another_binding_is_refused_before_its_decoder_runs() {
    let carried = <RefusalBinding as Declaration<PlanRefusal>>::declared_value(&refusal("e"));
    assert_eq!(
        carried.decode::<OtherBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::TypeMismatch {
            expected: "tests.other-refusal.v1",
            observed: "tests.plan-refusal.v1",
        })
    );
}

#[test]
fn an_undeclared_producer_is_reported_undeclared_not_empty() {
    let carried = <WorthQueryUndeclaredProducerDenial as Declaration<PlanRefusal>>::declared_value(
        &refusal("lost"),
    );
    assert_eq!(carried, WorthQueryProducerDenialValue::Undeclared);
    assert_eq!(carried.bytes(), None);
    assert_eq!(
        carried.decode::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::Undeclared)
    );
}

#[test]
fn a_value_the_binding_cannot_carry_is_reported_lost_never_truncated() {
    let carried =
        <RefusalBinding as Declaration<PlanRefusal>>::declared_value(&refusal(&"x".repeat(200)));
    let WorthQueryProducerDenialValue::EncodingRefused {
        value_identity,
        cause,
    } = &carried
    else {
        panic!("an oversized denial must not cross as bytes: {carried:?}");
    };
    assert_eq!(*value_identity, "tests.plan-refusal.v1");
    assert_eq!(carried.bytes(), None);
    assert_eq!(
        carried.decode::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::EncodingRefused(
            cause.clone()
        ))
    );
    assert!(matches!(
        carried.decode::<OtherBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::TypeMismatch { .. })
    ));
}

#[test]
fn forged_bytes_under_the_declared_identity_are_refused_as_invalid() {
    let oversized = WorthQueryProducerDenialValue::DeclaredValue {
        value_identity: "tests.plan-refusal.v1",
        bytes: vec![b'x'; 97],
    };
    assert_eq!(
        oversized.decode::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::InvalidEncoding)
    );
    let not_utf8 = WorthQueryProducerDenialValue::DeclaredValue {
        value_identity: "tests.plan-refusal.v1",
        bytes: vec![0xff, 0xfe],
    };
    assert_eq!(
        not_utf8.decode::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::InvalidEncoding)
    );
}

#[test]
fn installation_admits_only_finitely_bounded_declarations() {
    assert!(<RefusalBinding as Declaration<PlanRefusal>>::admits_installation());
    assert!(<WorthQueryUndeclaredProducerDenial as Declaration<
        PlanRefusal,
    >>::admits_installation());
    assert!(!<UnboundedBinding as Declaration<String>>::admits_installation());
    assert!(matches!(
        <UnboundedBinding as Declaration<String>>::declared_value(&"x".to_owned()),
        WorthQueryProducerDenialValue::EncodingRefused {
            cause: super::WorthQueryProducerDenialEncodingRefusal::NoFiniteWireBound,
            ..
        }
    ));
}

/// A producer whose denial is wider than the value its callers decode.
#[derive(Debug)]
enum FamilyDenial {
    Plan(PlanRefusal),
    MissingDefinition,
}
struct PlanEvidence;
impl WorthQueryProducerDenialProjection<FamilyDenial> for PlanEvidence {
    type Binding = RefusalBinding;
    fn project(denial: &FamilyDenial) -> Option<PlanRefusal> {
        match denial {
            FamilyDenial::Plan(refusal) => Some(refusal.clone()),
            FamilyDenial::MissingDefinition => None,
        }
    }
}
type Projected = WorthQueryProjectedProducerDenial<PlanEvidence>;

#[test]
fn a_projected_denial_carries_its_value_or_a_declared_absence() {
    let plan = refusal("entry_first_1 is degenerate");
    let carried =
        <Projected as Declaration<FamilyDenial>>::declared_value(&FamilyDenial::Plan(plan.clone()));
    assert_eq!(carried.value_identity(), Some("tests.plan-refusal.v1"));
    assert_eq!(carried.decode::<RefusalBinding>(), Ok(plan));
    let missing =
        <Projected as Declaration<FamilyDenial>>::declared_value(&FamilyDenial::MissingDefinition);
    assert_eq!(
        missing,
        WorthQueryProducerDenialValue::Absent {
            value_identity: "tests.plan-refusal.v1"
        }
    );
    assert_eq!(missing.bytes(), None);
    assert_eq!(
        missing.decode::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::Absent)
    );
    assert!(matches!(
        missing.decode::<OtherBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::TypeMismatch { .. })
    ));
    assert!(<Projected as Declaration<FamilyDenial>>::admits_installation());
}
