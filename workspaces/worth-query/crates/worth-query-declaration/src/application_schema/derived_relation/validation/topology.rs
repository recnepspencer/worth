//! Named recipe dependencies must form a DAG before any runtime is installed.
use super::*;

pub(in crate::application_schema::derived_relation) fn validate(
    definitions: &BTreeMap<&str, &ErasedApplicationDerivedRelationDefinition>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut remaining = BTreeMap::new();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (&name, definition) in definitions {
        let dependencies: BTreeSet<&str> = definition
            .parts()
            .expression
            .nodes
            .iter()
            .filter_map(|node| match node {
                ApplicationDerivedRelationNode::Source(source)
                    if source.kind == ApplicationDerivedRelationSourceKind::Derived =>
                {
                    Some(source.relation.as_str())
                }
                _ => None,
            })
            .collect();
        remaining.insert(name, dependencies.len());
        if dependencies.is_empty() {
            ready.insert(name);
        }
        for dependency in dependencies {
            dependents.entry(dependency).or_default().push(name);
        }
    }
    while let Some(name) = ready.pop_first() {
        if let Some(children) = dependents.get(name) {
            for child in children {
                let count = remaining.get_mut(child).expect("known derived dependency");
                *count -= 1;
                if *count == 0 {
                    ready.insert(*child);
                }
            }
        }
    }
    if let Some((&name, _)) = remaining.iter().find(|(_, count)| **count != 0) {
        return Err(denial(name, Cause::CyclicDefinitions));
    }
    Ok(())
}
