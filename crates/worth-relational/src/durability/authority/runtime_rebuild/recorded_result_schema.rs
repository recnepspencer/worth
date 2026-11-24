//! The result schema a durable schema-changing commit recorded for itself.
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::schema::data::SchemaAuthoritySnapshot;
use std::sync::Arc;

/// Replay of a recorded schema-changing commit judges it under the schema it
/// durably recorded, never under its predecessor's. Only recovery issues this,
/// after the carrier matched the authority in the commit's own envelope.
#[derive(Debug)]
pub(crate) struct RecordedReplayResultSchema {
    schema: Arc<RelationalBranchRootSchemaAuthority>,
}

impl RecordedReplayResultSchema {
    pub(super) fn admit(
        recorded: Arc<RelationalBranchRootSchemaAuthority>,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<Self, DurabilityError> {
        Self::admit_recorded(
            recorded,
            &envelope.schema_authority,
            envelope.commit.commit_id,
        )
    }

    /// `authority` is the schema authority the commit's own envelope recorded.
    fn admit_recorded(
        recorded: Arc<RelationalBranchRootSchemaAuthority>,
        authority: &SchemaAuthoritySnapshot,
        commit: CommitId,
    ) -> Result<Self, DurabilityError> {
        if !recorded.matches(authority) {
            return Err(DurabilityError::new(
                RecoveryFailureClass::SchemaMismatch,
                format!(
                    "recorded result schema of commit {} differs from its recorded authority",
                    commit.0
                ),
            ));
        }
        Ok(Self { schema: recorded })
    }

    pub(crate) fn into_schema(self) -> Arc<RelationalBranchRootSchemaAuthority> {
        self.schema
    }
}

#[cfg(test)]
mod tests {
    use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthorityFixturePort as _;

    use super::*;

    #[test]
    fn a_recorded_result_schema_admits_only_the_authority_its_envelope_recorded() {
        let recorded = RelationalBranchRootSchemaAuthority::empty_for_test();
        let own = crate::schema::data::RelationalSchemaRegistry::default().authority_snapshot();
        let admitted =
            RecordedReplayResultSchema::admit_recorded(recorded.clone(), &own, CommitId(1))
                .unwrap();
        assert!(Arc::ptr_eq(&admitted.into_schema(), &recorded));
        use crate::schema::data::{
            EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
        };
        let other = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: crate::identity::data::KindId(1),
                kind_name: "other-entity".into(),
                schema_id: SchemaId("other-schema".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap()
            .authority_snapshot();
        let refused =
            RecordedReplayResultSchema::admit_recorded(recorded, &other, CommitId(1)).unwrap_err();
        assert_eq!(refused.class, RecoveryFailureClass::SchemaMismatch);
    }
}
