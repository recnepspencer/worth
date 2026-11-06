//! Raw Native fixtures encode the installed record, including its structural columns.
use std::collections::BTreeMap;
use worth_query_decl::facade::application_schema::fragment::{AspectDefinition, Record};
pub fn values<Set: Record<T>, T: 'static>(
    locate: impl Fn(
        &worth_query_decl::facade::application_schema::ApplicationFieldBindingLocus,
    ) -> worth_foundational::facade::AspectFieldLocator,
    definition: &AspectDefinition,
    value: &T,
) -> BTreeMap<worth_foundational::facade::AspectFieldLocator, worth_foundational::facade::AspectValue>
{
    let mut encoded = definition
        .encode::<Set, T>(value)
        .expect("the installed row encodes");
    definition
        .fields()
        .iter()
        .filter_map(|field| {
            let value = encoded.remove(field.path())?;
            let locator = locate(field.recipe().locus());
            Some((locator.clone(), value))
        })
        .collect()
}
