//! Historical schema roles reserve identities even after a role is removed.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::original_inputs::CanonicalCommitEnvelopeOwnerPort as _;
use super::*;
use crate::identity::data::KindId;
use worth_foundational::facade::AspectIdentity;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

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
            for entry in (*registry.owner_entity_kinds()).values() {
                request.consult().map_err(Denial::Request)?;
                kind = kind.max(Some(entry.kind_id));
                for declaration in &entry.aspect_contract_declarations.aspects {
                    request.consult().map_err(Denial::Request)?;
                    aspect = aspect.max(Some(declaration.contract.identity()));
                }
            }
            for entry in (*registry.owner_relation_kinds()).values() {
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

    /// The maxima occupied when a recorded component was durable: every
    /// admitted schema except carriers first introduced by a later tail
    /// commit. A resumed transition recompiles its successor against these,
    /// never against identities its own durable prefix already introduced.
    pub fn occupied_identity_maxima_at(
        &self,
        request: &ExecutionRequest<'_, '_>,
        recorded: &crate::branch::recorded_basis::RecordedRelationalComponentReference,
    ) -> Result<(Option<KindId>, Option<AspectIdentity>), Denial> {
        let position = match recorded.target() {
            crate::branch::recorded_basis::RecordedRelationalComponentTarget::Canonical(source) => {
                Some(source.position())
            }
            crate::branch::recorded_basis::RecordedRelationalComponentTarget::Empty => None,
        };
        let plan = &self.state.plan;
        if plan
            .checkpoint_covered_position()
            .is_some_and(|covered| position.is_none_or(|position| covered > position))
        {
            return Err(Denial::PlanningPredecessorCovered);
        }
        let later = |commit: &crate::durability::migration::ReadmittedCanonicalCommit| {
            position.is_none_or(|position| commit.position() > position)
        };
        let mut present = std::collections::BTreeSet::new();
        let mut introduced_later = std::collections::BTreeSet::new();
        let mut predecessor_schema_seen = false;
        for commit in &plan.tail_log {
            request.consult().map_err(Denial::Request)?;
            let inputs = commit
                .envelope()
                .recorded_original_inputs()
                .ok_or(Denial::SchemaCarrierMissing)?;
            if !later(commit) {
                present.insert(inputs.prestate_schema);
                present.insert(inputs.result_schema);
            } else {
                // The first later commit starts from the predecessor's schema.
                if !predecessor_schema_seen {
                    present.insert(inputs.prestate_schema);
                    predecessor_schema_seen = true;
                }
                if inputs.result_image.is_some() {
                    introduced_later.insert(inputs.result_schema);
                }
            }
        }
        let mut kind = None;
        let mut aspect = None;
        for (digest, schema) in self.state.graph.schemas.rows.iter() {
            if introduced_later.contains(digest) && !present.contains(digest) {
                continue;
            }
            let registry = schema.registry();
            for (kind_id, aspects) in (*registry.owner_entity_kinds())
                .values()
                .map(|entry| (entry.kind_id, &entry.aspect_contract_declarations.aspects))
                .chain(
                    (*registry.owner_relation_kinds())
                        .values()
                        .map(|entry| (entry.kind_id, &entry.aspect_contract_declarations.aspects)),
                )
            {
                request.consult().map_err(Denial::Request)?;
                kind = kind.max(Some(kind_id));
                for declaration in aspects {
                    aspect = aspect.max(Some(declaration.contract.identity()));
                }
            }
        }
        Ok((kind, aspect))
    }
}
