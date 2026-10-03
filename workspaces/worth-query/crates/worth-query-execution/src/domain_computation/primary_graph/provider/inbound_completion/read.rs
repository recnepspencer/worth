//! Exact-basis indexed read of Query's canonical Relational completion row.

use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion,
    InternedString,
};
use worth_relational::facade::{
    history::{CommitId, RelationalCommitReceipt},
    runtime::RelationalRuntime,
};

use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;

#[cfg(test)]
mod lookup;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryInboundCompletionReadDenial {
    #[cfg(test)]
    SnapshotUnavailable,
    #[cfg(test)]
    IndexUnavailable,
    AmbiguousCorrelation,
    #[cfg(test)]
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
        _ => Err(Denial::Malformed),
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

#[cfg(test)]
mod provenance_tests {
    use super::lookup::hex;
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
