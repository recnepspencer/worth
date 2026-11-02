//! History metadata has its own closed installed slots, independent of payload masks.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationHistoryMetadataFacet, ApplicationHistoryMetadataMeaning,
    ApplicationHistoryQueryBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationHistoryQuery,
};

pub(in crate::domain_computation) fn compile_history_disclosure_contract<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
>(
    history: &WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
> {
    let definition = history.definition();
    let parts = definition.parts();
    definition.validate().map_err(|_| denial(&parts.name))?;
    let contract = history.metadata_disclosure();
    compile_metadata(&parts.history_type, contract, &parts.name)
}

pub(in crate::domain_computation) fn compile_document_history_disclosure_contract<S, H>(
    history: &worth_query_installation::facade::WorthQueryInstalledDocumentHistory<S, H>,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
>
where
    S: ApplicationSchema,
    H: worth_query_declaration::facade::application_history::ApplicationHistoryBinding<S>,
{
    let definition = history.definition();
    let parts = definition.parts();
    definition.validate().map_err(|_| denial(&parts.identity))?;
    compile_metadata(
        &parts.history_type,
        &parts.metadata_disclosure,
        &parts.identity,
    )
}

fn compile_metadata(
    history_type: &worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
    contract: &worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract,
    name: &str,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
> {
    contract
        .validate_history_metadata(history_type)
        .map_err(|_| denial(name))?;
    if contract.posture() == ApplicationQueryDisclosurePosture::Public {
        return Ok(WorthQueryAdmittedApplicationDisclosureContract::Public);
    }
    let mut result_rules = BTreeMap::new();
    for rule in contract.rules() {
        let ApplicationQueryDisclosureSelector::HistoryMetadata(metadata) = rule.selector() else {
            return Err(denial(name));
        };
        if metadata.history_identity() != *history_type
            || !rule.influence().permits(metadata.facet().influence())
        {
            return Err(denial(metadata.slot_type()));
        }
        let admitted = Arc::new(WorthQueryAdmittedApplicationDisclosureRule {
            disclosure_value: rule.disclosure_value().clone(),
            influence: rule.influence().clone(),
            field: None,
        });
        if result_rules.insert(metadata.slot_key(), admitted).is_some() {
            return Err(denial(metadata.slot_type()));
        }
    }
    if result_rules.len() != ApplicationHistoryMetadataFacet::ALL.len()
        || !ApplicationHistoryMetadataFacet::ALL
            .into_iter()
            .all(|facet| {
                result_rules.contains_key(
                    &ApplicationHistoryMetadataMeaning::from_untrusted_fields(
                        history_type.clone(),
                        facet,
                    )
                    .slot_key(),
                )
            })
    {
        return Err(denial(name));
    }
    Ok(WorthQueryAdmittedApplicationDisclosureContract::Governed {
        classification: contract.classification().to_owned(),
        capability_name: contract
            .capability_name()
            .ok_or_else(|| denial(name))?
            .to_owned(),
        capability_type: contract
            .capability_type()
            .ok_or_else(|| denial(name))?
            .to_owned(),
        result_rules,
        internal_rules: Vec::new(),
        internal_field_rules: BTreeMap::new(),
    })
}
