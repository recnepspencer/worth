//! Query-owned result interpretation consumes the exact Native presence outcome.
use super::{projection_denial, WorthQueryApplicationReadExecutionDenial};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldPresence as Presence;
use worth_relational::facade::runtime::RelationalProjectedFieldObservation as Observation;
pub(super) fn value<'a>(
    observed: Option<Observation<'a>>,
    presence: Presence,
    subject: &str,
) -> Result<Option<&'a AspectValue>, WorthQueryApplicationReadExecutionDenial> {
    match (observed, presence) {
        (Some(Observation::Present(value)), _) => Ok(Some(value)),
        (Some(Observation::AspectAbsent), Presence::OptionalAspect) => Ok(None),
        (Some(Observation::FieldAbsent | Observation::AspectAbsent), Presence::Optional) => {
            Ok(None)
        }
        _ => Err(projection_denial(subject)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_parent_result_accepts_only_genuine_parent_absence_for_required_child() {
        assert_eq!(
            value(
                Some(Observation::AspectAbsent),
                Presence::OptionalAspect,
                "bias"
            )
            .unwrap(),
            None
        );
        for observed in [
            None,
            Some(Observation::FieldAbsent),
            Some(Observation::UnsupportedAspectShape),
        ] {
            let denial = value(observed, Presence::OptionalAspect, "bias").unwrap_err();
            assert_eq!(denial.kind(), super::super::super::WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable);
        }
        assert!(value(Some(Observation::AspectAbsent), Presence::Required, "bias").is_err());
        assert!(value(Some(Observation::FieldAbsent), Presence::Required, "bias").is_err());
        assert_eq!(
            value(Some(Observation::FieldAbsent), Presence::Optional, "bias").unwrap(),
            None
        );
        let original = AspectValue::UInt64(7);
        assert_eq!(
            value(
                Some(Observation::Present(&original)),
                Presence::OptionalAspect,
                "bias"
            )
            .unwrap(),
            Some(&original)
        );
    }
}
