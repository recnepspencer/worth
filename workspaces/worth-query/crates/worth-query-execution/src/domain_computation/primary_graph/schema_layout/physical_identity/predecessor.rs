//! Original A role candidates are validated by A's normal compiler output.
use super::*;
impl PhysicalIdentityBindings {
    pub(in crate::domain_computation::primary_graph::schema_layout) fn readmit(
        schema: &ErasedApplicationSchemaDeclaration,
        contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
        source: &RelationalSchemaRegistry,
        maximum_kind: Option<KindId>,
        maximum_aspect: Option<AspectIdentity>,
    ) -> Result<Option<Self>, WorthQueryPrimaryGraphInstallationDenial> {
        // A real birth-only partial Home has no Query installation. Preserve
        // its existing missing-bootstrap owner path; this issues no old roles.
        let snapshot = source.authority_snapshot();
        if snapshot.entity_kinds.is_empty()
            && snapshot.relation_kinds.is_empty()
            && maximum_kind.is_none()
        {
            return Ok(None);
        }
        let (entities, relations) = source_application::resolve(
            schema,
            &super::super::registry_lowering::lower_application_contract_bindings(contracts),
            source,
            &snapshot,
        )?;
        Self::from_snapshot(
            source,
            &snapshot,
            entities,
            relations,
            maximum_kind,
            maximum_aspect,
        )
        .map(Some)
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn require_original(
        &self,
        compiled: &RelationalSchemaRegistry,
        source: &RelationalSchemaRegistry,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let expected = compiled.authority_snapshot();
        let actual = source.authority_snapshot();
        if expected.entity_kinds.len() != actual.entity_kinds.len()
            || expected.relation_kinds.len() != actual.relation_kinds.len()
        {
            return Err(invalid_member(
                "original physical role inventory differs from the installed meaning",
            ));
        }
        for kind in expected.entity_kinds {
            if compiled
                .entity_registration(kind.kind_id)
                .map_err(relational_schema_denial)?
                != source
                    .entity_registration(kind.kind_id)
                    .map_err(relational_schema_denial)?
            {
                return Err(invalid_member(
                    "original entity role differs from the installed meaning",
                ));
            }
        }
        for kind in expected.relation_kinds {
            if compiled
                .relation_registration(kind.kind_id)
                .map_err(relational_schema_denial)?
                != source
                    .relation_registration(kind.kind_id)
                    .map_err(relational_schema_denial)?
            {
                return Err(invalid_member(
                    "original relation role differs from the installed meaning",
                ));
            }
        }
        Ok(())
    }
}

/// Name/class select candidates; actual authored slots are already excluded.
/// The original-A compiler subsequently checks the complete private grammar.
pub(in crate::domain_computation::primary_graph::schema_layout) fn private_entity_role(
    registry: &RelationalSchemaRegistry,
    snapshot: &SchemaAuthoritySnapshot,
    application: &BTreeSet<KindId>,
    name: &str,
    aspect: &str,
) -> Result<(KindId, AspectIdentity), WorthQueryPrimaryGraphInstallationDenial> {
    let mut selected = None;
    for kind in snapshot
        .entity_kinds
        .iter()
        .filter(|kind| kind.kind_name == name && !application.contains(&kind.kind_id))
    {
        let registration = registry
            .entity_registration(kind.kind_id)
            .map_err(relational_schema_denial)?;
        for binding in &registration.aspect_contract_declarations.aspects {
            if binding.contract.key().as_str() == aspect {
                if selected
                    .replace((kind.kind_id, binding.contract.identity()))
                    .is_some()
                {
                    return Err(invalid_member("ambiguous original platform role"));
                }
            }
        }
    }
    selected.ok_or_else(|| invalid_member("original platform role is missing"))
}
