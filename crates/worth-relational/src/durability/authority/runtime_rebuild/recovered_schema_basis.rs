use std::collections::BTreeSet;

use crate::durability::data::{
    DurabilityError, RecoveryAuthorityContinuityMismatch, RecoveryFailureClass,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::mvcc::RelationalTransactionValidationInput;
use crate::schema::data::{
    DescriptorCanonicalBasisVersion, ProposedSchemaTransition, SchemaAuthoritySnapshot,
};
use crate::schema::{validate_schema_continuity_bundle, SchemaContinuityAuthorityInput};

pub(super) struct RecoveredSchemaBasis {
    authority_input: SchemaContinuityAuthorityInput,
    proposed_transition: Option<ProposedSchemaTransition>,
    reconciliation_policy: Option<crate::schema::data::SchemaReconciliationPolicy>,
    expected: ExpectedRecoveredSchema,
}

struct ExpectedRecoveredSchema {
    commit_id: u64,
    schema_version: crate::schema::data::SchemaVersionId,
    schema_authority: SchemaAuthoritySnapshot,
    descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    transition: Option<crate::schema::data::SchemaTransitionArtifact>,
    continuation: Option<crate::schema::data::SchemaContinuationDescriptor>,
    reconciliation: Option<crate::schema::data::SchemaReconciliationDescriptor>,
}

impl RecoveredSchemaBasis {
    pub(super) fn admit(
        runtime: &mut crate::runtime::RelationalRuntime,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<Self, DurabilityError> {
        validate_schema_continuity_bundle(envelope)
            .map_err(|issue| schema_mismatch(envelope, &issue.detail()))?;
        let schema_authority = canonical_authority_snapshot(envelope)?;
        let canonical_basis_version = canonical_basis_version(runtime, envelope)?;
        let proposed_transition =
            envelope
                .schema_transition
                .as_ref()
                .map(|transition| ProposedSchemaTransition {
                    source_schema_id: transition.source_schema_id.clone(),
                    source_schema_version_id: transition.source_schema_version_id,
                    target_schema_id: transition.target_schema_id.clone(),
                    target_schema_version_id: transition.target_schema_version_id,
                    diff_atoms: transition.diff_atoms.clone(),
                });
        let reconciliation_policy = envelope
            .schema_transition
            .as_ref()
            .map(|transition| transition.reconciliation_descriptor.policy);
        Ok(Self {
            authority_input: SchemaContinuityAuthorityInput::new(
                envelope.schema_version,
                schema_authority.clone(),
                envelope.descriptor_semantics_version,
                canonical_basis_version,
            ),
            proposed_transition,
            reconciliation_policy,
            expected: ExpectedRecoveredSchema {
                commit_id: envelope.commit.commit_id.0,
                schema_version: envelope.schema_version,
                schema_authority,
                descriptor_semantics_version: envelope.descriptor_semantics_version,
                transition: envelope.schema_transition.clone(),
                continuation: envelope.schema_continuation_descriptor.clone(),
                reconciliation: envelope.schema_reconciliation_descriptor.clone(),
            },
        })
    }

    pub(super) fn apply(
        &self,
        mut options: RelationalTransactionValidationInput,
    ) -> RelationalTransactionValidationInput {
        options = options.with_schema_authority_input(self.authority_input.clone());
        match self.proposed_transition.clone() {
            Some(transition) => {
                options.with_schema_transition(transition, self.reconciliation_policy)
            }
            None => options,
        }
    }

    pub(super) fn validate_replayed(
        &self,
        replayed: &CanonicalCommitEnvelope,
    ) -> Result<(), DurabilityError> {
        if replayed.schema_version != self.expected.schema_version
            || replayed.schema_authority != self.expected.schema_authority
        {
            return Err(schema_mismatch(
                replayed,
                "derived recovery schema snapshot differs from durable authority",
            ));
        }
        if replayed.descriptor_semantics_version != self.expected.descriptor_semantics_version {
            return Err(authority_mismatch(
                RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                    expected: self.expected.descriptor_semantics_version,
                    found: replayed.descriptor_semantics_version,
                },
            ));
        }
        if replayed.schema_transition != self.expected.transition {
            return Err(authority_mismatch(
                RecoveryAuthorityContinuityMismatch::SchemaTransitionArtifact {
                    commit_id: self.expected.commit_id,
                    detail: "normal derivation differs from durable transition".to_owned(),
                },
            ));
        }
        if replayed.schema_continuation_descriptor != self.expected.continuation {
            return Err(authority_mismatch(
                RecoveryAuthorityContinuityMismatch::ContinuationDescriptor {
                    commit_id: self.expected.commit_id,
                    boundary_fingerprint: self
                        .expected
                        .continuation
                        .as_ref()
                        .map(|descriptor| descriptor.boundary_fingerprint),
                    detail: "normal derivation differs from durable continuation".to_owned(),
                },
            ));
        }
        let expected_lineage = self
            .expected
            .reconciliation
            .as_ref()
            .map(|descriptor| &descriptor.resulting_lineage);
        let replayed_lineage = replayed
            .schema_reconciliation_descriptor
            .as_ref()
            .map(|descriptor| &descriptor.resulting_lineage);
        if replayed_lineage != expected_lineage {
            return Err(authority_mismatch(
                RecoveryAuthorityContinuityMismatch::SchemaLineage {
                    commit_id: self.expected.commit_id,
                    detail: "normal derivation differs from durable schema lineage".to_owned(),
                },
            ));
        }
        if replayed.schema_reconciliation_descriptor != self.expected.reconciliation {
            return Err(authority_mismatch(
                RecoveryAuthorityContinuityMismatch::ReconciliationDescriptor {
                    commit_id: self.expected.commit_id,
                    detail: "normal derivation differs from durable reconciliation".to_owned(),
                },
            ));
        }
        Ok(())
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
fn canonical_authority_snapshot(
    envelope: &CanonicalCommitEnvelope,
) -> Result<SchemaAuthoritySnapshot, DurabilityError> {
    let mut snapshot = envelope.schema_authority.clone();
    snapshot.entity_kinds.sort_by_key(|kind| kind.kind_id);
    snapshot.relation_kinds.sort_by_key(|kind| kind.kind_id);
    snapshot
        .derived_relations
        .sort_by(|left, right| left.identity.cmp(&right.identity));
    snapshot
        .projected_member_domains
        .sort_by(|left, right| left.identity.cmp(&right.identity));
    if snapshot != envelope.schema_authority {
        return Err(schema_mismatch(
            envelope,
            "schema authority rows are not canonical",
        ));
    }
    if snapshot.primary_schema_version_id != Some(envelope.schema_version) {
        return Err(schema_mismatch(
            envelope,
            "schema authority primary version differs from envelope version",
        ));
    }
    let mut definitions = BTreeSet::new();
    let mut domains = BTreeSet::new();
    for domain in &snapshot.projected_member_domains {
        if !domains.insert(&domain.identity)
            || snapshot.primary_schema_id.as_ref() != Some(&domain.identity.schema)
            || snapshot.primary_schema_version_id != Some(domain.identity.version)
            || domain.cases.is_empty()
            || domain.cases.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(schema_mismatch(
                envelope,
                "schema authority contains noncanonical or mixed-basis projected domains",
            ));
        }
    }
    for definition in &snapshot.derived_relations {
        if !definitions.insert(&definition.identity)
            || snapshot.primary_schema_id.as_ref() != Some(&definition.identity.schema)
            || snapshot.primary_schema_version_id != Some(definition.identity.version)
        {
            return Err(schema_mismatch(
                envelope,
                "schema authority contains duplicate or mixed-basis derived definitions",
            ));
        }
    }
    let mut kind_ids = BTreeSet::new();
    for (kind_id, schema_id, schema_version) in snapshot
        .entity_kinds
        .iter()
        .map(|kind| (kind.kind_id, &kind.schema_id, kind.schema_version_id))
        .chain(
            snapshot
                .relation_kinds
                .iter()
                .map(|kind| (kind.kind_id, &kind.schema_id, kind.schema_version_id)),
        )
    {
        if !kind_ids.insert(kind_id)
            || snapshot.primary_schema_id.as_ref() != Some(schema_id)
            || snapshot.primary_schema_version_id != Some(schema_version)
        {
            return Err(schema_mismatch(
                envelope,
                "schema authority contains duplicate or mixed-basis kind rows",
            ));
        }
    }
    Ok(snapshot)
}

fn canonical_basis_version(
    runtime: &mut crate::runtime::RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
) -> Result<DescriptorCanonicalBasisVersion, DurabilityError> {
    match (
        envelope.schema_continuation_descriptor.as_ref(),
        envelope.schema_reconciliation_descriptor.as_ref(),
    ) {
        (Some(continuation), Some(reconciliation))
            if continuation.bridge.canonical_basis_version
                == reconciliation.canonical_basis_version =>
        {
            Ok(continuation.bridge.canonical_basis_version)
        }
        (None, None) => Ok(runtime
            .config
            .schema
            .descriptor_canonical_basis_policy
            .current_write_version()),
        _ => Err(schema_mismatch(
            envelope,
            "schema descriptors disagree on canonical basis version",
        )),
    }
}

fn schema_mismatch(envelope: &CanonicalCommitEnvelope, detail: &str) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::SchemaMismatch,
        format!(
            "recovered schema basis denied at commit {}: {detail}",
            envelope.commit.commit_id.0
        ),
    )
}

fn authority_mismatch(mismatch: RecoveryAuthorityContinuityMismatch) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::SchemaMismatch,
        "recovered schema derivation mismatch",
    )
    .with_authority_continuity_mismatch(mismatch)
}
