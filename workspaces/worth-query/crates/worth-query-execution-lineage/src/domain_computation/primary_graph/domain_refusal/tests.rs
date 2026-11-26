//! The refusal grammar, its bounds and every decoder that must re-admit it.
use super::*;
use crate::domain_computation::primary_graph::{HandlerResult, WorthQueryOperationBodyDenial};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
};
use WorthQueryDomainRefusalDenial as Denial;

const CODE_MAX: usize = WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES;
const MESSAGE_MAX: usize = WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES;

fn refusal(code: &str, message: &str) -> Result<WorthQueryDomainRefusal, Denial> {
    WorthQueryDomainRefusal::new(code, message)
}

#[test]
fn an_admitted_refusal_keeps_its_exact_code_and_message() {
    let lot = refusal(
        "stock.lot_expired-2",
        "Lot 7 expired on 3 May — it can’t ship.",
    )
    .unwrap();
    assert_eq!(lot.code(), "stock.lot_expired-2");
    assert_eq!(lot.message(), "Lot 7 expired on 3 May — it can’t ship.");
    assert_eq!(lot.to_string(), lot.message());
    let longest = refusal(&"a".repeat(CODE_MAX), &"é".repeat(MESSAGE_MAX / 2)).unwrap();
    assert_eq!(
        (longest.code().len(), longest.message().len()),
        (CODE_MAX, MESSAGE_MAX)
    );
}

#[test]
fn each_grammar_violation_is_refused_with_its_own_typed_reason() {
    let cases: [(&str, &str, Denial); 9] = [
        ("", "ok", Denial::CodeEmpty),
        (
            &"a".repeat(CODE_MAX + 1),
            "ok",
            Denial::CodeTooLong {
                maximum: CODE_MAX,
                observed: CODE_MAX + 1,
            },
        ),
        ("Lot_expired", "ok", Denial::CodeCharacter { position: 0 }),
        ("9lives", "ok", Denial::CodeCharacter { position: 0 }),
        ("lot expired", "ok", Denial::CodeCharacter { position: 3 }),
        ("lot_é", "ok", Denial::CodeCharacter { position: 4 }),
        ("lot", " \u{3000} ", Denial::MessageBlank),
        (
            "lot",
            "first\nsecond",
            Denial::MessageControlCharacter { position: 5 },
        ),
        (
            "lot",
            &"x".repeat(MESSAGE_MAX + 1),
            Denial::MessageTooLong {
                maximum: MESSAGE_MAX,
                observed: MESSAGE_MAX + 1,
            },
        ),
    ];
    for (code, message, expected) in cases {
        assert_eq!(
            refusal(code, message),
            Err(expected),
            "{code:?} / {message:?}"
        );
    }
    // An oversized code is refused by length before its characters are read.
    assert_eq!(
        refusal(&"X".repeat(1 << 20), "ok"),
        Err(Denial::CodeTooLong {
            maximum: CODE_MAX,
            observed: 1 << 20
        })
    );
}

#[test]
fn deserialization_re_admits_the_grammar() {
    let lot = refusal("lot_expired", "Lot 7 expired.").unwrap();
    let json = serde_json::to_string(&lot).unwrap();
    assert_eq!(
        serde_json::from_str::<WorthQueryDomainRefusal>(&json).unwrap(),
        lot
    );
    let forged = [
        r#"{"code":"Lot","message":"x"}"#,
        r#"{"code":"lot","message":"a\nb"}"#,
        r#"{"code":"lot","message":"x","extra":1}"#,
    ];
    for text in forged {
        assert!(
            serde_json::from_str::<WorthQueryDomainRefusal>(text).is_err(),
            "{text}"
        );
    }
    let denial = WorthQueryOperationBodyDenial::DomainRefused(lot.clone());
    let json = serde_json::to_string(&denial).unwrap();
    let back: WorthQueryOperationBodyDenial = serde_json::from_str(&json).unwrap();
    assert_eq!(back.domain_refusal(), Some(&lot));
}

#[test]
fn the_canonical_binding_round_trips_the_widest_refusal_and_refuses_forged_text() {
    let widest = refusal(&"z".repeat(CODE_MAX), &"\"".repeat(MESSAGE_MAX)).unwrap();
    let encoded = WorthQueryDomainRefusalBinding::encode(&widest).unwrap();
    assert_eq!(
        WorthQueryDomainRefusalBinding::decode(&encoded).unwrap(),
        widest
    );

    let text = WorthQueryDomainRefusalBinding::encode(&refusal("lot", "ok").unwrap())
        .unwrap()
        .raw_text()
        .unwrap()
        .to_owned();
    let forged = AspectValue::String(text.replace("\"lot\"", "\"Lot\"").into());
    assert!(WorthQueryDomainRefusalBinding::decode(&forged).is_err());
}

#[test]
fn a_handler_refusal_is_a_domain_denial_and_a_bad_reason_is_an_execution_defect() {
    type Result = HandlerResult<(), WorthQueryOperationBodyDenial>;
    let HandlerResult::DomainDenied(denial) = Result::domain_refusal("lot_expired", "Expired.")
    else {
        panic!("an admitted reason is a domain denial");
    };
    assert_eq!(
        denial.domain_refusal().map(WorthQueryDomainRefusal::code),
        Some("lot_expired")
    );
    let HandlerResult::ExecutionDenied(defect) = Result::domain_refusal("lot", "") else {
        panic!("an inadmissible reason is an execution defect");
    };
    assert_eq!(defect.downcast::<Denial>().ok(), Some(Denial::MessageBlank));
}
