//! Read-only DATA slots after the sole durable intent grammar validates them.
use super::{shape, MALFORMED};
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RecordedIntentObservation {
    pub(in crate::domain_computation::primary_graph) root: [u8; 32],
    pub(in crate::domain_computation::primary_graph) correction: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) conditional: Option<[u8; 32]>,
    mutation: Option<[u8; 32]>,
}
impl RecordedIntentObservation {
    /// The root names the binding's canonical input. The optional `input`
    /// slot names separately governed admission input and may be absent/different.
    pub(in crate::domain_computation::primary_graph) fn matches_mutation_input(
        &self,
        name: &str,
        input: &[u8; 32],
    ) -> bool {
        self.root == *input && self.names_mutation(name)
    }
    pub(in crate::domain_computation::primary_graph) fn names_mutation(&self, name: &str) -> bool {
        self.mutation == Some(super::super::mutation_binding::slot(name))
    }
}
pub(super) fn read(text: &str) -> Result<RecordedIntentObservation, &'static str> {
    shape::validate(text)?;
    let mut slots = text.strip_prefix("v2:").unwrap_or(text).split(':');
    let root = decode(slots.next().ok_or(MALFORMED)?)?;
    let mut observed = RecordedIntentObservation {
        root,
        correction: None,
        conditional: None,
        mutation: None,
    };
    for slot in slots {
        let (name, value) = slot.split_once('=').ok_or(MALFORMED)?;
        if value == "-" {
            continue;
        }
        match name {
            "correction" => observed.correction = Some(decode(value)?),
            "conditional-definition" => observed.conditional = Some(decode(value)?),
            "mutation-binding" => observed.mutation = Some(decode(value)?),
            "input"
            | "source"
            | "operation"
            | "scope"
            | "precondition"
            | "proposal"
            | "workflow-definition"
            | "workflow-instance"
            | "workflow-proposal-context"
            | "workflow-captured-inputs"
            | "workflow-transition"
            | "workflow-support"
            | "workflow-client-key"
            | "workflow-approval"
            | "edit-envelope"
            | "transport-intent" => {}
            _ => return Err(MALFORMED),
        }
    }
    Ok(observed)
}
fn decode(value: &str) -> Result<[u8; 32], &'static str> {
    if value.len() != 64 {
        return Err(MALFORMED);
    }
    let mut decoded = [0; 32];
    for (byte, pair) in decoded.iter_mut().zip(value.as_bytes().as_chunks::<2>().0) {
        let digit = |byte| match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            _ => Err(MALFORMED),
        };
        *byte = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests;
