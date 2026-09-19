use super::WorthQueryPrimaryGraph;

impl WorthQueryPrimaryGraph {
    pub(in crate::domain_computation) fn registered_provider_aspect_identities(
        &self,
    ) -> [worth_foundational::facade::AspectIdentity; 3] {
        let kinds = [
            self.layout.provider_idempotency().entity_kind,
            self.layout.provider_dispatch_outbox().entity_kind,
            self.layout.provider_aftermath_causality().entity_kind,
        ];
        self.source_owner.with_runtime(|runtime| {
            kinds.map(|kind| {
                runtime
                    .config()
                    .schema
                    .registry
                    .entity_registration(kind)
                    .expect("provider entity is registered")
                    .aspect_contract_declarations
                    .aspects
                    .first()
                    .expect("provider entity has its contract")
                    .contract
                    .identity()
            })
        })
    }

    pub(in crate::domain_computation) fn registered_entity_aspect(
        &self,
        entity: &str,
        aspect: &str,
    ) -> Option<worth_relational::facade::schema::DeclaredAspectContractBinding> {
        let kind = self.layout.entity_kind(entity)?;
        self.source_owner.with_runtime(|runtime| {
            runtime
                .config()
                .schema
                .registry
                .entity_registration(kind)
                .ok()?
                .aspect_contract_declarations
                .aspects
                .iter()
                .find(|declared| declared.contract.key().as_str() == aspect)
                .cloned()
        })
    }
}
