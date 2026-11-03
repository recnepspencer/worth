use super::{planned_field_locator, WorthQueryPrimaryGraphInstallationDenial};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectContract, AspectFieldLocator};
use worth_query_installation::facade::{
    ApplicationFieldBindingOwnerKind, WorthQueryInstalledApplicationSchemaContractCatalog,
};

#[derive(Debug, Default)]
pub(super) struct ApplicationRelationPayloadLayout {
    relations: BTreeMap<String, BTreeMap<String, RelationAspect>>,
}
#[derive(Debug)]
struct RelationAspect {
    contract: AspectContract,
    fields: BTreeMap<String, AspectFieldLocator>,
}
impl ApplicationRelationPayloadLayout {
    pub(super) fn lower(
        catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let mut relations = BTreeMap::<String, BTreeMap<String, RelationAspect>>::new();
        for installed in catalog.contracts().filter(|value| {
            value.locus().owner_kind() == ApplicationFieldBindingOwnerKind::Relation
        }) {
            let locus = installed.locus();
            let fields = installed
                .fields()
                .map(|field| {
                    Ok((
                        field.as_str().to_owned(),
                        planned_field_locator(locus.aspect().as_str(), field.as_str())?,
                    ))
                })
                .collect::<Result<_, WorthQueryPrimaryGraphInstallationDenial>>()?;
            relations
                .entry(locus.record_name().to_owned())
                .or_default()
                .insert(
                    locus.aspect().as_str().to_owned(),
                    RelationAspect {
                        contract: installed.contract().clone(),
                        fields,
                    },
                );
        }
        Ok(Self { relations })
    }
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn relation_field_locator(
        &self,
        relation: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectFieldLocator> {
        self.relation_payload
            .relations
            .get(relation)?
            .get(aspect)?
            .fields
            .get(field)
    }
    pub(in crate::domain_computation::primary_graph) fn relation_aspect_contract(
        &self,
        relation: &str,
        aspect: &str,
    ) -> Option<&AspectContract> {
        Some(
            &self
                .relation_payload
                .relations
                .get(relation)?
                .get(aspect)?
                .contract,
        )
    }
}
