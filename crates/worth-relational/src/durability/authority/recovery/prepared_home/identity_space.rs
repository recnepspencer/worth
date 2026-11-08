//! Historical schema roles reserve identities even after a role is removed.
use super::*;
use crate::identity::data::KindId;
use worth_foundational::facade::AspectIdentity;

impl PreparedRelationalHomeRecovery {
    /// DATA only: the maxima cover every admitted original schema, including
    /// historical prestate/result images, not merely the selected receiver.
    pub fn occupied_identity_maxima(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(Option<KindId>, Option<AspectIdentity>), Denial> {
        let mut kind = None;
        let mut aspect = None;
        for schema in self.state.graph.schemas.rows.values() {
            let registry = schema.registry();
            for entry in registry.entity_kinds.values() {
                request.consult().map_err(Denial::Request)?;
                kind = kind.max(Some(entry.kind_id));
                for declaration in &entry.aspect_contract_declarations.aspects {
                    request.consult().map_err(Denial::Request)?;
                    aspect = aspect.max(Some(declaration.contract.identity()));
                }
            }
            for entry in registry.relation_kinds.values() {
                request.consult().map_err(Denial::Request)?;
                kind = kind.max(Some(entry.kind_id));
                for declaration in &entry.aspect_contract_declarations.aspects {
                    request.consult().map_err(Denial::Request)?;
                    aspect = aspect.max(Some(declaration.contract.identity()));
                }
            }
        }
        Ok((kind, aspect))
    }
}
