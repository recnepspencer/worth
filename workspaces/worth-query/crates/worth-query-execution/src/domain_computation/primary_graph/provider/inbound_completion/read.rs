//! Exact-basis indexed read of Query's canonical Relational completion row.

use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion,
    InternedString,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{CommitId, RelationalCommitReceipt},
    indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime},
    storage::RecordLifecycleState,
};

use super::super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryInboundCompletionLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryInboundCompletionReadDenial {
    SnapshotUnavailable,
    IndexUnavailable,
    AmbiguousCorrelation,
    RowUnavailable,
    Malformed,
    CommitUnavailable,
    ReconstructionWorkExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCanonicalCompletionRow {
    pub correlation: ExternalEffectCorrelationIdentity,
    pub family: String,
    pub operation: String,
    pub audience: String,
    pub source: String,
    pub expires_at: Option<u64>,
    pub key_epoch: Option<u64>,
    pub message_identity: Option<[u8; 32]>,
    pub protocol_identity: BoundaryProtocolIdentity,
    pub protocol_version: BoundaryProtocolVersion,
    pub signed_meaning_digest: Option<[u8; 32]>,
    pub provenance: WorthQueryCompletionProvenance,
    pub payload: Vec<u8>,
    pub original_commit: RelationalCommitReceipt,
    pub original_incarnation_ordinal: u64,
    pub completion_commit: RelationalCommitReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryCompletionProvenance {
    AuthenticatedInbound,
    InstalledTransportCompletion {
        attempt_identity: [u8; 32],
        observation_identity: [u8; 32],
    },
}

impl WorthQueryPrimaryGraphProvider {
    /// A correlation is selected through the Relational owner's exact index
    /// generation. Missing is authoritative only for this admitted basis.
    pub(in crate::domain_computation::primary_graph) fn lookup_inbound_completion_row(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial>
    {
        let layout = self.graph.layout.provider_inbound_completion().clone();
        self.graph
            .with_runtime_mut(|runtime| read_at_basis(runtime, basis, &layout, correlation))
    }
}

fn read_at_basis(
    runtime: &mut RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    layout: &WorthQueryInboundCompletionLayout,
    correlation: &ExternalEffectCorrelationIdentity,
) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    let snapshot =
        crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime, basis,
        )
        .map_err(|_| Denial::SnapshotUnavailable)?;
    let result = read_at_snapshot(runtime, &snapshot, layout, correlation);
    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
    result
}

fn read_at_snapshot(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryInboundCompletionLayout,
    correlation: &ExternalEffectCorrelationIdentity,
) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    let value = AspectValue::String(InternedString::from(hex(correlation.bytes())));
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        layout.correlation_index_id,
        layout.kind,
        layout.correlation.clone(),
        value,
        2,
    )
    .map_err(|_| Denial::IndexUnavailable)?;
    let lookup = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|_| Denial::IndexUnavailable)?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(Denial::AmbiguousCorrelation);
    }
    let Some(entity_id) = lookup.candidate_entity_ids().first().copied() else {
        return Ok(None);
    };
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
    let fields = locators
        .iter()
        .map(|locator| field(locator))
        .collect::<Result<Vec<_>, _>>()?;
    let aspect = layout.correlation.aspect().aspect_key().clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        aspect.clone(),
        fields.clone(),
    )]);
    let (created_at, values) = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .and_then(|view| {
            view.entity_record_with_projection_scope(entity_id, scope, |record| {
                (record.kind_id() == layout.kind
                    && record.lifecycle() == RecordLifecycleState::Live)
                    .then(|| {
                        (
                            record.created_at_version(),
                            fields
                                .iter()
                                .map(|field| record.aspect_field_value(&aspect, field).cloned())
                                .collect::<Option<Vec<_>>>(),
                        )
                    })
            })
        })
        .ok_or(Denial::RowUnavailable)?;
    let values = values.ok_or(Denial::Malformed)?;
    let completed = runtime
        .history()
        .historical_committed_version(created_at)
        .ok_or(Denial::CommitUnavailable)?
        .commit()
        .clone();
    let row = decode_completion_values(runtime, &values, completed)?;
    if row.correlation != *correlation {
        return Err(Denial::Malformed);
    }
    Ok(Some(row))
}

pub(super) fn decode_completion_values(
    runtime: &RelationalRuntime,
    values: &[AspectValue],
    completed: RelationalCommitReceipt,
) -> Result<WorthQueryCanonicalCompletionRow, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    if values.len() != 20 {
        return Err(Denial::Malformed);
    }
    let original_id = number(&values[12])?;
    let original = runtime
        .history()
        .immutable_commit_receipt(CommitId(original_id))
        .ok_or(Denial::CommitUnavailable)?;
    if original.branch_id.0 != raw_string(&values[13])? {
        return Err(Denial::Malformed);
    }
    let row_correlation = digest(&values[0])?;
    let version = u32::try_from(number(&values[9])?).map_err(|_| Denial::Malformed)?;
    let (provenance, expires_at, key_epoch, message_identity, signed_meaning_digest) =
        decode_provenance_fields(values)?;
    let marker = raw_string(&values[16])?;
    if !matches!(
        (&provenance, marker.as_str()),
        (
            WorthQueryCompletionProvenance::AuthenticatedInbound,
            "consumed-completed"
        ) | (
            WorthQueryCompletionProvenance::InstalledTransportCompletion { .. },
            "installed-transport-completed"
        )
    ) {
        return Err(Denial::Malformed);
    }
    Ok(WorthQueryCanonicalCompletionRow {
        correlation: row_correlation,
        family: raw_string(&values[1])?,
        operation: raw_string(&values[2])?,
        audience: raw_string(&values[3])?,
        source: raw_string(&values[4])?,
        expires_at,
        key_epoch,
        message_identity,
        protocol_identity: BoundaryProtocolIdentity::parse(raw_string(&values[8])?)
            .map_err(|_| Denial::Malformed)?,
        protocol_version: BoundaryProtocolVersion::try_new(version)
            .map_err(|_| Denial::Malformed)?,
        signed_meaning_digest,
        provenance,
        payload: decode_hex(&raw_string(&values[11])?)?,
        original_commit: original,
        original_incarnation_ordinal: number(&values[15])?,
        completion_commit: completed,
    })
}

type DecodedProvenance = (
    WorthQueryCompletionProvenance,
    Option<u64>,
    Option<u64>,
    Option<[u8; 32]>,
    Option<[u8; 32]>,
);

fn decode_provenance_fields(
    values: &[AspectValue],
) -> Result<DecodedProvenance, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    if values.len() != 20 {
        return Err(Denial::Malformed);
    }
    match raw_string(&values[17])?.as_str() {
        "authenticated-inbound" => {
            if !raw_string(&values[18])?.is_empty() || !raw_string(&values[19])?.is_empty() {
                return Err(Denial::Malformed);
            }
            Ok((
                WorthQueryCompletionProvenance::AuthenticatedInbound,
                Some(number(&values[5])?),
                Some(number(&values[6])?),
                Some(digest_bytes(&values[7])?),
                Some(digest_bytes(&values[10])?),
            ))
        }
        "installed-transport-completion" => {
            if number(&values[5])? != 0
                || number(&values[6])? != 0
                || !raw_string(&values[7])?.is_empty()
                || !raw_string(&values[10])?.is_empty()
            {
                return Err(Denial::Malformed);
            }
            Ok((
                WorthQueryCompletionProvenance::InstalledTransportCompletion {
                    attempt_identity: digest_bytes(&values[18])?,
                    observation_identity: digest_bytes(&values[19])?,
                },
                None,
                None,
                None,
                None,
            ))
        }
        _ => return Err(Denial::Malformed),
    }
}

pub(super) fn field(
    locator: &AspectFieldLocator,
) -> Result<worth_foundational::facade::FieldKey, WorthQueryInboundCompletionReadDenial> {
    locator
        .field_path()
        .fields()
        .first()
        .cloned()
        .ok_or(WorthQueryInboundCompletionReadDenial::Malformed)
}

fn raw_string(value: &AspectValue) -> Result<String, WorthQueryInboundCompletionReadDenial> {
    match value {
        AspectValue::String(InternedString::Raw(value)) => Ok(value.clone()),
        _ => Err(WorthQueryInboundCompletionReadDenial::Malformed),
    }
}

fn number(value: &AspectValue) -> Result<u64, WorthQueryInboundCompletionReadDenial> {
    match value {
        AspectValue::UInt64(value) => Ok(*value),
        _ => Err(WorthQueryInboundCompletionReadDenial::Malformed),
    }
}

fn digest(
    value: &AspectValue,
) -> Result<ExternalEffectCorrelationIdentity, WorthQueryInboundCompletionReadDenial> {
    let bytes = digest_bytes(value)?;
    Ok(ExternalEffectCorrelationIdentity::from_digest(
        worth_foundational::facade::CanonicalDigestId::new(bytes),
    ))
}

fn digest_bytes(value: &AspectValue) -> Result<[u8; 32], WorthQueryInboundCompletionReadDenial> {
    decode_hex(&raw_string(value)?)?
        .try_into()
        .map_err(|_| WorthQueryInboundCompletionReadDenial::Malformed)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    if !value.len().is_multiple_of(2) || !value.is_ascii() {
        return Err(Denial::Malformed);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(
                std::str::from_utf8(pair).map_err(|_| Denial::Malformed)?,
                16,
            )
            .map_err(|_| Denial::Malformed)
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn text(value: &str) -> AspectValue {
        AspectValue::String(InternedString::from(value.to_owned()))
    }

    #[test]
    fn transport_row_cannot_claim_a_signed_message_or_key_epoch() {
        let mut values = vec![text(""); 20];
        values[5] = AspectValue::UInt64(0);
        values[6] = AspectValue::UInt64(0);
        values[17] = text("installed-transport-completion");
        values[18] = text(&hex(&[0x31; 32]));
        values[19] = text(&hex(&[0x41; 32]));
        assert!(matches!(
            decode_provenance_fields(&values),
            Ok((
                WorthQueryCompletionProvenance::InstalledTransportCompletion { .. },
                None,
                None,
                None,
                None
            ))
        ));
        values[7] = text(&hex(&[0x51; 32]));
        assert_eq!(
            decode_provenance_fields(&values),
            Err(WorthQueryInboundCompletionReadDenial::Malformed)
        );
        values[7] = text("");
        values[6] = AspectValue::UInt64(1);
        assert_eq!(
            decode_provenance_fields(&values),
            Err(WorthQueryInboundCompletionReadDenial::Malformed)
        );
        values[6] = AspectValue::UInt64(0);
        values[10] = text(&hex(&[0x61; 32]));
        assert_eq!(
            decode_provenance_fields(&values),
            Err(WorthQueryInboundCompletionReadDenial::Malformed)
        );
    }
}
