//! The durable intent recorded under an idempotency key, and how a request's
//! intent compares with a record written by this or the first encoding.
//!
//! The current encoding opens with `v2:` and names only identities that survive
//! a restore or reopen. The first encoding opened directly with the intent
//! identity and also named the runtime that admitted the request: its scope
//! slot led with the runtime-authority and installation-runtime ordinals, and
//! its operation slot held the operation's authority seal, keyed per
//! installation. A first-encoding record resolves by its durable parts. Every
//! slot other than the operation must equal the request's, and the operation
//! must follow from the mutation binding the record names within the package
//! and schema its scope names. A record that names an operation but no mutation
//! binding cannot confirm that operation, so it is unverifiable, never drift.

use super::WorthQueryApplicationIdempotencyBinding;

const DURABLE_ENCODING: &str = "v2:";
const OPERATION_SLOT: &str = ":operation=";
const SCOPE_SLOT: &str = ":scope=";
const IDENTITY_DIGITS: usize = 64;
/// The digits of the two runtime ordinals that lead a first-encoding scope.
const FIRST_ENCODING_RUNTIME_DIGITS: usize = 32;
const MALFORMED: &str = "provider idempotency intent is malformed";

/// How a request's intent compares with the intent recorded under its key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryRecordedIntentMatch {
    /// The record commits this intent.
    Same,
    /// The record commits a different intent.
    Drift,
    /// The record's durable parts match, but it names its operation only by a
    /// seal no later runtime can confirm.
    Unverifiable,
}

impl WorthQueryApplicationIdempotencyBinding {
    /// The intent this binding records under its key.
    pub(in crate::domain_computation::primary_graph) fn intent_text(self) -> String {
        format!("{DURABLE_ENCODING}{}", self.intent_slots())
    }

    pub(in crate::domain_computation::primary_graph) fn match_recorded_intent(
        self,
        recorded: &str,
    ) -> Result<WorthQueryRecordedIntentMatch, &'static str> {
        let slots = self.intent_slots();
        if let Some(recorded) = recorded.strip_prefix(DURABLE_ENCODING) {
            return Ok(if recorded == slots {
                WorthQueryRecordedIntentMatch::Same
            } else {
                WorthQueryRecordedIntentMatch::Drift
            });
        }
        let (recorded, recorded_operation) =
            without_operation(&without_runtime_ordinals(recorded)?)?;
        let (requested, requested_operation) = without_operation(&slots)?;
        if recorded != requested || recorded_operation != requested_operation {
            return Ok(WorthQueryRecordedIntentMatch::Drift);
        }
        let operation_follows =
            self.mutation_binding_identity.is_some() && self.operation_scope_identity.is_some();
        Ok(if !requested_operation || operation_follows {
            WorthQueryRecordedIntentMatch::Same
        } else {
            WorthQueryRecordedIntentMatch::Unverifiable
        })
    }
}

/// A first-encoding intent with the runtime ordinals removed from its scope.
fn without_runtime_ordinals(recorded: &str) -> Result<String, &'static str> {
    let identity = recorded.get(..IDENTITY_DIGITS).ok_or(MALFORMED)?;
    if !is_hex(identity) || !recorded[IDENTITY_DIGITS..].starts_with(':') {
        return Err(MALFORMED);
    }
    let scope = recorded.find(SCOPE_SLOT).ok_or(MALFORMED)? + SCOPE_SLOT.len();
    if recorded[scope..].starts_with('-') {
        return Ok(recorded.to_owned());
    }
    let ordinals = recorded
        .get(scope..scope + FIRST_ENCODING_RUNTIME_DIGITS)
        .filter(|ordinals| is_hex(ordinals))
        .ok_or(MALFORMED)?;
    Ok(format!(
        "{}{}",
        &recorded[..scope],
        &recorded[scope + ordinals.len()..]
    ))
}

/// The intent with its operation slot's value removed, and whether the slot
/// named an operation.
fn without_operation(slots: &str) -> Result<(String, bool), &'static str> {
    let start = slots.find(OPERATION_SLOT).ok_or(MALFORMED)? + OPERATION_SLOT.len();
    let rest = &slots[start..];
    let end = rest.find(':').unwrap_or(rest.len());
    let named = match &rest[..end] {
        "-" => false,
        value if value.len() == IDENTITY_DIGITS && is_hex(value) => true,
        _ => return Err(MALFORMED),
    };
    Ok((format!("{}{}", &slots[..start], &rest[end..]), named))
}

fn is_hex(digits: &str) -> bool {
    digits
        .bytes()
        .all(|digit| matches!(digit, b'0'..=b'9' | b'a'..=b'f'))
}
