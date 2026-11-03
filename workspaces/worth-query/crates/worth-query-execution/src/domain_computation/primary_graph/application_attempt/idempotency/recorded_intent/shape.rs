//! The borrowed grammar shared by durable recovery and request comparison.
use super::{is_hex, MALFORMED};
pub(in crate::domain_computation::primary_graph) fn validate(
    input: &str,
) -> Result<(), &'static str> {
    let (input, scope_digits) = match input.strip_prefix("v2:") {
        Some(input) => (input, 192),
        None => (input, 240),
    };
    let mut slots = input.split(':');
    let root = slots.next().ok_or(MALFORMED)?;
    if root.len() != 64 || !is_hex(root) {
        return Err(MALFORMED);
    }
    for name in [
        "source",
        "operation",
        "scope",
        "precondition",
        "input",
        "proposal",
        "conditional-definition",
    ] {
        let slot = slots.next().ok_or(MALFORMED)?;
        let (actual, value) = slot.split_once('=').ok_or(MALFORMED)?;
        let digits = if name == "scope" { scope_digits } else { 64 };
        if actual != name || !(value == "-" || value.len() == digits && is_hex(value)) {
            return Err(MALFORMED);
        }
    }
    let optional = [
        "workflow-definition",
        "workflow-instance",
        "workflow-proposal-context",
        "workflow-captured-inputs",
        "workflow-transition",
        "workflow-support",
        "workflow-client-key",
        "workflow-approval",
        "mutation-binding",
        "correction",
        "transport-intent",
    ];
    let mut previous = None;
    for slot in slots {
        let (name, value) = slot.split_once('=').ok_or(MALFORMED)?;
        let index = optional
            .iter()
            .position(|candidate| *candidate == name)
            .ok_or(MALFORMED)?;
        if previous.is_some_and(|previous| previous >= index) || value.len() != 64 || !is_hex(value)
        {
            return Err(MALFORMED);
        }
        previous = Some(index);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durable_intent_shape_refuses_unknown_reordered_and_nullable_optional_slots() {
        let base = format!("v2:{}:source=-:operation=-:scope=-:precondition=-:input=-:proposal=-:conditional-definition=-", "01".repeat(32));
        assert!(validate(&base).is_ok());
        assert!(validate(&format!("{base}:mutation-binding={}", "02".repeat(32))).is_ok());
        for suffix in [
            ":mutation-binding=-",
            ":unknown=-",
            ":",
            ":correction=00:mutation-binding=00",
        ] {
            assert!(validate(&format!("{base}{suffix}")).is_err());
        }
        assert!(validate(&base.replace(":source=-:operation=-", ":operation=-:source=-")).is_err());
        let digest = "02".repeat(32);
        assert!(validate(&format!(
            "{base}:correction={digest}:mutation-binding={digest}"
        ))
        .is_err());
        assert!(validate(&format!(
            "{base}:mutation-binding={digest}:mutation-binding={digest}"
        ))
        .is_err());
        assert!(validate(&format!("{base}:mutation-binding={}", "AB".repeat(32))).is_err());
        assert!(validate(&base.replacen("01", "0A", 1)).is_err());
        assert!(validate(&base.replacen(":input=-", "", 1)).is_err());
    }

    #[test]
    fn durable_and_legacy_scopes_and_complete_optional_suffix_are_exact() {
        let digest = "01".repeat(32);
        for (prefix, width) in [("v2:", 192), ("", 240)] {
            let scope = "a".repeat(width);
            let base = format!("{prefix}{digest}:source={digest}:operation={digest}:scope={scope}:precondition={digest}:input={digest}:proposal={digest}:conditional-definition={digest}");
            assert!(validate(&base).is_ok());
            assert!(validate(&base.replace(&format!("scope={scope}"), "scope=-")).is_ok());
            assert!(validate(&base.replace(&scope, &"a".repeat(432 - width))).is_err());
            let suffix = [
                "workflow-definition",
                "workflow-instance",
                "workflow-proposal-context",
                "workflow-captured-inputs",
                "workflow-transition",
                "workflow-support",
                "workflow-client-key",
                "workflow-approval",
                "mutation-binding",
                "correction",
                "transport-intent",
            ]
            .map(|name| format!(":{name}={digest}"))
            .concat();
            assert!(validate(&format!("{base}{suffix}")).is_ok());
        }
    }
}
