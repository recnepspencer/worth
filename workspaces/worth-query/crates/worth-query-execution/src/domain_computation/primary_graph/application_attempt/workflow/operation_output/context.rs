//! Persisted output-context shape, not occurrence or performed-receipt authority.
use super::super::schema::version::{
    WORKFLOW_FACT_PROTOCOL_VERSION, WORKFLOW_OPERATION_OUTPUT_PROTOCOL_VERSION,
};
use worth_foundational::facade::{AspectValue, InternedString};

pub(in crate::domain_computation::primary_graph) fn decode_transition_output_context(
    protocol: Option<AspectValue>,
    binding: Option<AspectValue>,
    count: Option<AspectValue>,
) -> Result<Option<(String, usize)>, ()> {
    match (protocol, binding, count) {
        // Old missing context is never backfilled from today's selection.
        (Some(AspectValue::UInt64(WORKFLOW_FACT_PROTOCOL_VERSION)), None, None) => Ok(None),
        (
            Some(AspectValue::UInt64(WORKFLOW_OPERATION_OUTPUT_PROTOCOL_VERSION)),
            Some(AspectValue::String(InternedString::Raw(binding))),
            Some(AspectValue::UInt64(count)),
        ) if !binding.is_empty() => Ok(Some((binding, usize::try_from(count).map_err(|_| ())?))),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(value: &str) -> Option<AspectValue> {
        Some(AspectValue::String(InternedString::Raw(value.into())))
    }
    #[test]
    fn transition_output_context_accepts_legacy_absence_and_zero_output_profiles() {
        assert_eq!(
            decode_transition_output_context(Some(AspectValue::UInt64(2)), None, None),
            Ok(None)
        );
        assert_eq!(
            decode_transition_output_context(
                Some(AspectValue::UInt64(3)),
                text("binding"),
                Some(AspectValue::UInt64(0))
            ),
            Ok(Some(("binding".into(), 0)))
        );
    }
    #[test]
    fn transition_output_context_refuses_unknown_or_malformed_profiles() {
        for (protocol, binding, count) in [
            (
                Some(AspectValue::UInt64(4)),
                text("binding"),
                Some(AspectValue::UInt64(0)),
            ),
            (None, None, None),
            (text("3"), text("binding"), Some(AspectValue::UInt64(0))),
            (
                Some(AspectValue::UInt64(3)),
                None,
                Some(AspectValue::UInt64(0)),
            ),
            (
                Some(AspectValue::UInt64(3)),
                text(""),
                Some(AspectValue::UInt64(0)),
            ),
            (
                Some(AspectValue::UInt64(3)),
                Some(AspectValue::UInt64(1)),
                Some(AspectValue::UInt64(0)),
            ),
            (Some(AspectValue::UInt64(3)), text("binding"), None),
            (Some(AspectValue::UInt64(3)), text("binding"), text("0")),
            (
                Some(AspectValue::UInt64(2)),
                text("binding"),
                Some(AspectValue::UInt64(0)),
            ),
        ] {
            assert!(decode_transition_output_context(protocol, binding, count).is_err());
        }
    }
}
