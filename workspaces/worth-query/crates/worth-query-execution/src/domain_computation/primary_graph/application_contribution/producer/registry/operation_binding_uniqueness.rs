use std::collections::BTreeMap;

use super::DeclaredProducerBinding;

pub(super) fn duplicate_operation_binding(
    declared: &BTreeMap<String, DeclaredProducerBinding>,
) -> Option<(&str, &str)> {
    let declarations = declared.values().collect::<Vec<_>>();
    for (index, left) in declarations.iter().enumerate() {
        if let Some(right) = declarations[index + 1..]
            .iter()
            .find(|right| left.operation_binding_type == right.operation_binding_type)
        {
            return Some((left.identity.as_str(), right.identity.as_str()));
        }
    }
    None
}
