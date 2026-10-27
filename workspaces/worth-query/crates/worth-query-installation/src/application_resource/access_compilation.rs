use super::{
    WorthQueryApplicationResourceInstallationDenial as Denial,
    WorthQueryApplicationResourceInstallationDenialKind as Kind,
    WorthQueryInstalledResourceAccessContract as Contract,
    WorthQueryResourceAccessPurpose as Purpose,
};
use crate::{
    application_operation::WorthQueryInstalledAbilityRequirement,
    application_schema::WorthQueryInstalledApplicationSchema,
    graph_obligation::{bind_resource_obligations, install_disclosure_capabilities},
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureSelector},
    application_resource::ErasedApplicationResourceDefinition,
    application_schema::ApplicationSchema,
};

pub(super) fn compile_access_contracts<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    definition: &ErasedApplicationResourceDefinition,
    requirements: &BTreeMap<
        worth_query_declaration::facade::application_resource::ApplicationResourceAction,
        WorthQueryInstalledAbilityRequirement,
    >,
) -> Result<BTreeMap<Purpose, Arc<Contract>>, Denial> {
    let parts = definition.parts();
    let mut contracts = BTreeMap::new();
    for purpose in Purpose::ALL {
        let capabilities = if purpose.facets().is_empty()
            || parts.disclosure.posture() == ApplicationQueryDisclosurePosture::Public
        {
            Vec::new()
        } else {
            let actual = install_disclosure_capabilities(schema, &parts.disclosure);
            if actual.is_empty() {
                return Err(Denial::new(Kind::AccessContractInvalid, &parts.name));
            }
            actual
        };
        let rules = parts
            .disclosure
            .rules()
            .iter()
            .filter(|rule| {
                matches!(rule.selector(), ApplicationQueryDisclosureSelector::Resource(meaning)
                if meaning.resource_identity() == parts.resource_type
                && purpose.facets().contains(&meaning.facet()))
            })
            .cloned()
            .collect();
        let obligations = bind_resource_obligations(
            &schema.binding_identity(),
            &parts.name,
            purpose,
            requirements.get(&purpose.action()),
            &capabilities,
        )
        .map_err(|_| Denial::new(Kind::AccessContractInvalid, &parts.name))?;
        contracts.insert(
            purpose,
            Arc::new(Contract {
                purpose,
                obligations: Arc::new(obligations),
                rules,
                slots: purpose.facets().iter().map(|facet| {
                    worth_query_declaration::facade::application_resource::ApplicationResourceDisclosureMeaning::from_untrusted_fields(parts.resource_type.clone(), *facet).slot_key()
                }).collect(),
            }),
        );
    }
    Ok(contracts)
}
