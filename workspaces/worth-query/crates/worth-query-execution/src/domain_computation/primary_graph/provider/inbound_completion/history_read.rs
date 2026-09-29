//! Bounded canonical patch join for World Performed completion history.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use worth_foundational::facade::{AspectValue, FieldKey};
use worth_relational::facade::history::{
    BoundedCanonicalCommitPatchDenial, RelationalCommitIdentity,
};
use worth_relational::facade::publication::{PublishedAuthoritativePatch, RecordStructuralChange};

use super::read::{decode_completion_values, field, WorthQueryCanonicalCompletionRow};
use super::WorthQueryInboundCompletionReadDenial as Denial;
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryInboundCompletionLayout;

impl WorthQueryPrimaryGraphProvider {
    /// Inspect only the changed records in one independently World-paired
    /// Relational commit. Every record consumes caller-declared work.
    pub(in crate::domain_computation::primary_graph) fn completion_row_from_commit_patch(
        &self,
        identity: &RelationalCommitIdentity,
        maximum_changed_records: NonZeroUsize,
    ) -> Result<Option<WorthQueryCanonicalCompletionRow>, Denial> {
        let layout = self.graph.layout.provider_inbound_completion().clone();
        self.graph.with_runtime(|runtime| {
            let receipt = runtime
                .history()
                .immutable_commit_receipt(identity.commit_id())
                .ok_or(Denial::CommitUnavailable)?;
            if receipt.version_id != identity.version_id()
                || &receipt.branch_id != identity.authoring_branch()
            {
                return Err(Denial::CommitUnavailable);
            }
            let patches = runtime
                .history()
                .bounded_canonical_commit_patches(identity.commit_id(), maximum_changed_records)
                .map_err(|denial| match denial {
                    BoundedCanonicalCommitPatchDenial::CommitUnavailable => {
                        Denial::CommitUnavailable
                    }
                    BoundedCanonicalCommitPatchDenial::WorkExhausted { .. } => {
                        Denial::ReconstructionWorkExhausted
                    }
                })?;
            let mut found = None;
            for patch in patches {
                if patch.structural_change != RecordStructuralChange::Created
                    || !matches!(
                        patch.target,
                        worth_relational::facade::transactions::RecordRef::Entity(_)
                    )
                {
                    continue;
                }
                let Some(values) = completion_patch_values(&layout, &patch.authoritative_patch)?
                else {
                    continue;
                };
                let row = decode_completion_values(runtime, &values, receipt.clone())?;
                if found.replace(row).is_some() {
                    return Err(Denial::AmbiguousCorrelation);
                }
            }
            Ok(found)
        })
    }
}

fn completion_patch_values(
    layout: &WorthQueryInboundCompletionLayout,
    patch: &PublishedAuthoritativePatch,
) -> Result<Option<Vec<AspectValue>>, Denial> {
    let aspect = layout.correlation.aspect().aspect_key();
    let whole = patch.struct_set_for(aspect);
    let field_sets = patch.field_sets_for(aspect).collect::<Vec<_>>();
    if whole.is_none() && field_sets.is_empty() {
        return Ok(None);
    }
    let mut values = BTreeMap::<FieldKey, AspectValue>::new();
    if let Some(whole) = whole {
        values.extend(
            whole
                .fields()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }
    for set in field_sets {
        if values
            .insert(set.field.clone(), set.value.clone())
            .is_some()
        {
            return Err(Denial::Malformed);
        }
    }
    let locators = [
        &layout.correlation,
        &layout.family,
        &layout.operation,
        &layout.audience,
        &layout.source,
        &layout.expires_at,
        &layout.key_epoch,
        &layout.message,
        &layout.protocol,
        &layout.version,
        &layout.meaning_digest,
        &layout.payload,
        &layout.original_commit,
        &layout.original_branch,
        &layout.original_entity,
        &layout.original_incarnation,
        &layout.terminal,
        &layout.provenance_kind,
        &layout.transport_attempt,
        &layout.transport_observation,
    ];
    locators
        .iter()
        .map(|locator| {
            let key = field(locator)?;
            values.get(&key).cloned().ok_or(Denial::Malformed)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
