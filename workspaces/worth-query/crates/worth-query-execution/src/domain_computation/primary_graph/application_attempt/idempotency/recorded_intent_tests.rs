//! A recorded intent matches a request by its durable parts, in the current
//! encoding and in the first one, which also named the admitting runtime.

use super::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryIdempotencyEntityIdentity,
    WorthQueryIdempotencyScopeIdentity, WorthQueryRecordedIntentMatch as Match,
};

const DEFINITION: [u8; 32] = [0x0d; 32];
const SEAL: [u8; 32] = [0x5e; 32];

#[test]
fn the_recorded_intent_names_its_encoding_and_no_runtime() {
    let first = mutation_request(3);
    let later = mutation_request(3);

    assert!(first.intent_text().starts_with("v2:"));
    assert_eq!(first.intent_text(), later.intent_text());
    assert_eq!(
        first.match_recorded_intent(&later.intent_text()),
        Ok(Match::Same)
    );
    assert_eq!(
        first.match_recorded_intent(&mutation_request(4).intent_text()),
        Ok(Match::Drift)
    );
}

#[test]
fn a_first_encoding_record_resolves_by_its_durable_parts() {
    let request = mutation_request(3);
    for ordinals in [
        "00000000000000010000000000000001",
        "0000000000000007000000000000002a",
    ] {
        assert_eq!(
            request.match_recorded_intent(&first_encoding(mutation_request(3), ordinals)),
            Ok(Match::Same),
            "the runtime that admitted the request does not decide its intent"
        );
    }
}

#[test]
fn a_first_encoding_record_with_another_durable_part_is_drift() {
    let request = mutation_request(3);
    let ordinals = "00000000000000010000000000000001";
    assert_eq!(
        request.match_recorded_intent(&first_encoding(mutation_request(4), ordinals)),
        Ok(Match::Drift),
        "another governed input"
    );
    let mut other_principal = mutation_request(3);
    other_principal.operation_scope_identity = Some(scope(11));
    assert_eq!(
        request.match_recorded_intent(&first_encoding(other_principal, ordinals)),
        Ok(Match::Drift),
        "another principal"
    );
    let mut other_binding = mutation_request(3);
    other_binding.mutation_binding_identity = Some([0x6c; 32]);
    assert_eq!(
        request.match_recorded_intent(&first_encoding(other_binding, ordinals)),
        Ok(Match::Drift),
        "another mutation binding"
    );
}

#[test]
fn a_first_encoding_operation_without_a_mutation_binding_is_unverifiable() {
    let mut request = mutation_request(3);
    request.mutation_binding_identity = None;
    let ordinals = "00000000000000010000000000000001";

    assert_eq!(
        request.match_recorded_intent(&first_encoding(request, ordinals)),
        Ok(Match::Unverifiable)
    );
}

#[test]
fn a_first_encoding_record_without_operation_or_scope_matches_exactly() {
    let request = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_governed_input(Some(&[3; 32]));
    let recorded = request.intent_slots();

    assert_eq!(request.match_recorded_intent(&recorded), Ok(Match::Same));
    assert_eq!(
        request
            .bind_governed_input(Some(&[4; 32]))
            .match_recorded_intent(&recorded),
        Ok(Match::Drift)
    );
}

#[test]
fn a_record_in_neither_encoding_is_malformed_not_drift() {
    let request = mutation_request(3);
    for recorded in ["", "v1:00", "zz", &request.intent_slots()[..70]] {
        assert!(
            request.match_recorded_intent(recorded).is_err(),
            "{recorded:?}"
        );
    }
}

/// A request to a mutation binding, admitted for one principal and scope.
fn mutation_request(input: u8) -> WorthQueryApplicationIdempotencyBinding {
    let mut binding = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_operation(&DEFINITION)
        .bind_governed_input(Some(&[input; 32]));
    binding.operation_scope_identity = Some(scope(10));
    binding.mutation_binding_identity = Some([0x6b; 32]);
    binding
}

/// The intent the first encoding recorded for `request`: its operation slot
/// held the installation-keyed seal, and its scope led with the two runtime
/// ordinals.
fn first_encoding(request: WorthQueryApplicationIdempotencyBinding, ordinals: &str) -> String {
    let slots = request.bind_operation(&SEAL).intent_slots();
    let scope = slots.find(":scope=").unwrap() + ":scope=".len();
    format!("{}{ordinals}{}", &slots[..scope], &slots[scope..])
}

fn scope(principal_slot: u64) -> WorthQueryIdempotencyScopeIdentity {
    WorthQueryIdempotencyScopeIdentity {
        binding_generation: 5,
        package_identity: [6; 32],
        schema_identity: [7; 32],
        principal: WorthQueryIdempotencyEntityIdentity {
            partition: 8,
            local_slot: principal_slot,
            generation: 9,
        },
        scope: WorthQueryIdempotencyEntityIdentity {
            partition: 8,
            local_slot: 20,
            generation: 9,
        },
    }
}
