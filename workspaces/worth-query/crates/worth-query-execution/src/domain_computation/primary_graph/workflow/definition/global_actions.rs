//! Versioned retained meaning shared by cold execution and adoption inventory.
use worth_query_declaration::facade::{
    application_operation::ApplicationWorkflowExecutionRequirements,
    application_program::ApplicationWorkflowGlobalAction,
};
use worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency as Dependency;

pub(super) fn encode(actions: &[ApplicationWorkflowGlobalAction]) -> String {
    serde_json::json!([
        1,
        actions
            .iter()
            .map(|action| {
                let operation = action.operation();
                serde_json::json!([
                    1,
                    operation.identifier(),
                    operation.input_type().as_str(),
                    operation.binding().expect("validated global binding").0,
                    operation
                        .workflow_execution_requirements()
                        .persistence_identity()
                ])
            })
            .collect::<Vec<_>>()
    ])
    .to_string()
}

pub(super) fn decode(
    encoded: &str,
    maximum_actions: usize,
    maximum_canonical_bytes: usize,
) -> Option<Box<[Dependency]>> {
    // JSON escaping can expand canonical text by at most six bytes per byte.
    // Bound parsing/allocation before decoding the count and strings.
    if encoded.len() > maximum_canonical_bytes.saturating_mul(6) {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(encoded).ok()?;
    let envelope = value.as_array()?;
    if envelope.len() != 2 || envelope[0].as_u64()? != 1 {
        return None;
    }
    let records = envelope[1].as_array()?;
    if records.is_empty() || records.len() > maximum_actions {
        return None;
    }
    let mut actions = Vec::with_capacity(records.len());
    let mut bindings = std::collections::BTreeSet::new();
    for record in records {
        let fields = record.as_array()?;
        if fields.len() != 5 || fields[0].as_u64()? != 1 {
            return None;
        }
        let text = |index: usize| {
            fields[index]
                .as_str()
                .filter(|text| !text.trim().is_empty())
        };
        let identifier = text(1)?.to_owned();
        let input_type = text(2)?.to_owned();
        let binding = text(3)?.to_owned();
        let requirements =
            ApplicationWorkflowExecutionRequirements::from_persistence_identity(text(4)?)?;
        if requirements.approval().is_some() || !bindings.insert(binding.clone()) {
            return None;
        }
        actions.push(Dependency::Terminate {
            identifier,
            input_type,
            binding,
            requirements,
        });
    }
    Some(actions.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_global_action_codec_rejects_unknown_shape_duplicate_bindings_and_resource_excess() {
        let record = serde_json::json!([
            1,
            "End",
            "end-input",
            "end-binding",
            ApplicationWorkflowExecutionRequirements::occurrence_only().persistence_identity()
        ]);
        let valid = serde_json::json!([1, [record.clone()]]);
        let encoded = valid.to_string();
        let decoded = decode(&encoded, 1, 4096).unwrap();
        assert_eq!(decoded[0].binding(), "end-binding");
        assert!(decode(&encoded, 0, 4096).is_none());
        assert!(decode(&encoded, 1, 1).is_none());
        assert!(decode(
            &serde_json::json!([1, [record.clone(), record.clone()]]).to_string(),
            2,
            4096
        )
        .is_none());
        for invalid in [
            serde_json::json!([2, [record.clone()]]),
            serde_json::json!([1, [record.clone()], 7]),
            serde_json::json!([1, []]),
        ] {
            assert!(decode(&invalid.to_string(), 1, 4096).is_none());
        }
        for (index, value) in [
            (0, serde_json::json!(2)),
            (1, serde_json::json!("")),
            (3, serde_json::json!("")),
            (4, serde_json::json!("unknown-floor")),
        ] {
            let mut changed = record.clone();
            changed[index] = value;
            assert!(decode(&serde_json::json!([1, [changed]]).to_string(), 1, 4096).is_none());
        }
    }
}
