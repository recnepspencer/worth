mod bindings;

pub use bindings::*;

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectValue, InternedString, StoredValueText, Symbol};
    use worth_query_decl::facade::application_schema::ApplicationReadableScalarValueBinding;

    use super::{EstateCaseStatusBinding, EstateWorkflowStageBinding};
    use crate::estate::{EstateCaseStatus, EstateWorkflowStage};

    #[test]
    fn estate_enum_bindings_read_stored_text_and_refuse_invalid_tags() {
        let stored = |text: &str| {
            AspectValue::String(InternedString::Stored(StoredValueText::new(
                text.to_owned(),
            )))
        };
        assert_eq!(
            EstateWorkflowStageBinding::decode(&stored("accounts-frozen")).unwrap(),
            EstateWorkflowStage::AccountsFrozen
        );
        assert_eq!(
            EstateCaseStatusBinding::decode(&stored("open")).unwrap(),
            EstateCaseStatus::Open
        );
        assert!(EstateWorkflowStageBinding::decode(&stored("unknown-stage")).is_err());
        assert!(EstateCaseStatusBinding::decode(&stored("unknown-status")).is_err());
        assert!(
            EstateWorkflowStageBinding::decode(&AspectValue::String(InternedString::Symbol(
                Symbol(7)
            )))
            .is_err()
        );
    }
}
