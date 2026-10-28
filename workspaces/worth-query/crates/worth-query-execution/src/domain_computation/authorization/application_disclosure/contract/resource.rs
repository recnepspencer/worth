//! Compile exactly the outputs of one installed Resource action surface.
use super::*;
use worth_query_declaration::facade::application_resource::{
    ApplicationResourceBinding, ApplicationResourceDisclosureMeaning,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationResource, WorthQueryResourceAccessPurpose,
};

pub(in crate::domain_computation) fn compile_resource_disclosure_contract<Schema, Binding>(
    resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
    purpose: WorthQueryResourceAccessPurpose,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let definition = resource.definition();
    let parts = definition.parts();
    definition.validate().map_err(|_| denial(&parts.name))?;
    let contract = &parts.disclosure;
    if purpose.facets().is_empty()
        || contract.posture() == ApplicationQueryDisclosurePosture::Public
    {
        return Ok(WorthQueryAdmittedApplicationDisclosureContract::Public);
    }
    if contract.posture() != ApplicationQueryDisclosurePosture::Governed {
        return Err(denial(&parts.name));
    }
    let mut result_rules = BTreeMap::new();
    for rule in resource.access_contract(purpose).disclosure_rules() {
        let ApplicationQueryDisclosureSelector::Resource(meaning) = rule.selector() else {
            return Err(denial(&parts.name));
        };
        if meaning.resource_identity() != parts.resource_type
            || !purpose.facets().contains(&meaning.facet())
            || !meaning
                .facet()
                .required_influences()
                .iter()
                .all(|surface| rule.influence().permits(*surface))
        {
            return Err(denial(meaning.slot_type()));
        }
        let admitted = Arc::new(WorthQueryAdmittedApplicationDisclosureRule {
            disclosure_value: rule.disclosure_value().clone(),
            influence: rule.influence().clone(),
            field: None,
        });
        if result_rules.insert(meaning.slot_key(), admitted).is_some() {
            return Err(denial(meaning.slot_type()));
        }
    }
    if result_rules.len() != purpose.facets().len()
        || !purpose.facets().iter().all(|facet| {
            result_rules.contains_key(
                &ApplicationResourceDisclosureMeaning::from_untrusted_fields(
                    parts.resource_type.clone(),
                    *facet,
                )
                .slot_key(),
            )
        })
    {
        return Err(denial(&parts.name));
    }
    Ok(WorthQueryAdmittedApplicationDisclosureContract::Governed {
        classification: contract.classification().to_owned(),
        capability_name: contract
            .capability_name()
            .ok_or_else(|| denial(&parts.name))?
            .to_owned(),
        capability_type: contract
            .capability_type()
            .ok_or_else(|| denial(&parts.name))?
            .to_owned(),
        result_rules,
        internal_rules: Vec::new(),
        internal_field_rules: BTreeMap::new(),
    })
}
