//! Navigation idempotency identities stay byte-identical across the S4 key change.
//!
//! Navigation admission hashes `(Operation::IDENTIFIER, key)` and
//! `(Operation::IDENTIFIER, N::IDENTITY)` through `for_host_commit`.
//! `bind_admission` never changes either identity, and the navigation kind is
//! not hashed, so undo, redo_to and checkout share these bytes. The envelope,
//! correction and group constants live in `golden/history_identity.json`
//! (captured from master, `logs/tbu-2e-a/s0a-golden-capture.log`). The
//! envelope and correction rows are checked against the production hasher
//! here; the group rows are data until S4.
use super::*;
use worth_query_decl::facade::application_history::ApplicationHistoryNavigationBinding;
use worth_query_host::facade::application_entry::history_envelope_identities_for_test;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::navigation::Navigation;

type Operation = <Navigation as ApplicationHistoryNavigationBinding<
    DocumentRetentionSchema,
    RetentionHistory,
>>::Operation;

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

const KEY_IDENTITIES: [(u128, &str); 3] = [
    (
        0,
        "daae9476a2de380f63c983f17d84866cffecb43a5e1c5ac35df0b2d6a9eef58e",
    ),
    (
        1,
        "2b6d96e71449ed940ba8f9aa6c2aeed65ceef2079324beb162d3a5916dc9ea12",
    ),
    (
        u128::MAX,
        "312c6f39ae4e4e24f550d5d0e2c6ea3bfd4c79aa22dc6929852c5b5aabb60b51",
    ),
];
const INTENT_IDENTITY: &str = "1d6a0e729072b32999868dc615ad6d731f7e69b396eba7c7b9aa2b47c6a33307";

#[test]
fn u128_navigation_identities_match_the_master_golden() {
    let intent = <Navigation as ApplicationHistoryNavigationBinding<
        DocumentRetentionSchema,
        RetentionHistory,
    >>::IDENTITY;
    for (key, expected) in KEY_IDENTITIES {
        let binding = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            DocumentRetentionSchema,
            Operation,
            _,
            _,
        >(&key, intent)
        .expect("a u128 key and a static intent encode");
        assert_eq!(hex(binding.key_identity()), expected, "key {key:#x}");
        assert_eq!(
            hex(binding.intent_identity()),
            INTENT_IDENTITY,
            "key {key:#x}"
        );
    }
}

fn bytes(text: &serde_json::Value) -> [u8; 32] {
    let text = text.as_str().expect("a hex identity");
    std::array::from_fn(|at| u8::from_str_radix(&text[2 * at..2 * at + 2], 16).expect("hex"))
}

#[test]
fn envelope_and_correction_identities_match_the_master_golden() {
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("golden/history_identity.json")).unwrap();
    let rows = golden["envelope"].as_array().unwrap();
    assert!(rows.iter().any(|row| row.get("correction").is_some()));
    for row in rows {
        let correction = row.get("correction").map(|correction| {
            (
                correction["binding"].as_str().unwrap(),
                correction["source_ordinal"].as_u64().unwrap(),
                correction["member"].as_str().unwrap(),
            )
        });
        let (key, intent) = history_envelope_identities_for_test(
            row["history"].as_str().unwrap(),
            &bytes(&row["key"]),
            &bytes(&row["edit_intent"]),
            correction,
        );
        assert_eq!(hex(&key), row["key_identity"].as_str().unwrap(), "{row}");
        assert_eq!(
            hex(&intent),
            row["intent_identity"].as_str().unwrap(),
            "{row}"
        );
    }
}
