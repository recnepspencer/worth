//! Concrete bounded encoding for original accepted message meaning.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceClaims;
use worth_foundational::facade::BoundaryProtocolIdentity;

pub(super) fn bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_le_bytes());
    output.extend_from_slice(value);
}
pub(super) fn text(output: &mut Vec<u8>, value: &str) {
    bytes(output, value.as_bytes());
}
fn key(output: &mut Vec<u8>, value: &QueryInboundMessageKey) {
    text(output, &value.audience);
    text(output, &value.protocol);
    text(output, &value.source);
    output.extend_from_slice(&value.epoch.to_le_bytes());
    output.extend_from_slice(&value.message);
}
pub(in crate::domain_computation::application_aftermath::dispatch_journal) fn encode(
    output: &mut Vec<u8>,
    value: &QueryInboundRecord,
) {
    match value {
        QueryInboundRecord::TransportBinding(binding) => {
            output.push(6);
            output.extend_from_slice(binding.correlation.bytes());
            text(output, &binding.operation);
            match &binding.audience {
                None => output.push(0),
                Some(audience) => {
                    output.push(1);
                    text(output, audience);
                }
            }
            super::source::encode(output, &binding.source);
        }
        QueryInboundRecord::Accepted(accepted) => {
            output.push(3);
            text(output, &accepted.operation);
            let claims = &accepted.claims;
            key(output, &QueryInboundMessageKey::from_claims(claims));
            output.extend_from_slice(&claims.signed_meaning_digest);
            output.extend_from_slice(&claims.issued_at_unix_seconds.to_le_bytes());
            output.extend_from_slice(&claims.expires_at_unix_seconds.to_le_bytes());
            output.extend_from_slice(&claims.protocol_version.get().to_le_bytes());
            text(output, &claims.correlation_family);
            output.extend_from_slice(&claims.correlation_token);
            bytes(output, &claims.payload);
            output.extend_from_slice(&accepted.envelope_bytes.to_le_bytes());
            super::source::encode(output, &accepted.source);
        }
        QueryInboundRecord::Acknowledged {
            message,
            completion_world,
        } => {
            output.push(4);
            key(output, message);
            output.extend_from_slice(&completion_world.to_le_bytes());
        }
        QueryInboundRecord::Expired { message } => {
            output.push(5);
            key(output, message);
        }
    }
}

impl Reader<'_> {
    pub(super) fn number(&mut self) -> Result<u64, QueryDispatchJournalDenial> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    pub(super) fn blob(&mut self) -> Result<Vec<u8>, QueryDispatchJournalDenial> {
        let length = usize::try_from(self.number()?)
            .map_err(|_| QueryDispatchJournalDenial::MalformedFrame)?;
        let (head, rest) = self
            .0
            .split_at_checked(length)
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
        self.0 = rest;
        Ok(head.to_vec())
    }
    pub(super) fn text(&mut self) -> Result<String, QueryDispatchJournalDenial> {
        String::from_utf8(self.blob()?).map_err(|_| QueryDispatchJournalDenial::MalformedFrame)
    }
    fn message_key(&mut self) -> Result<QueryInboundMessageKey, QueryDispatchJournalDenial> {
        Ok(QueryInboundMessageKey {
            audience: self.text()?,
            protocol: self.text()?,
            source: self.text()?,
            epoch: self.number()?,
            message: self.take()?,
        })
    }
}
pub(super) fn decode(
    tag: u8,
    reader: &mut Reader<'_>,
) -> Result<QueryInboundRecord, QueryDispatchJournalDenial> {
    use QueryDispatchJournalDenial::MalformedFrame;
    Ok(match tag {
        3 => {
            let operation = reader.text()?;
            let message = reader.message_key()?;
            let claims = WorthQueryInboundOccurrenceClaims {
                audience: message.audience,
                protocol_identity: BoundaryProtocolIdentity::parse(message.protocol)
                    .map_err(|_| MalformedFrame)?,
                source_identity: message.source,
                key_epoch: message.epoch,
                message_identity: message.message,
                signed_meaning_digest: reader.take()?,
                issued_at_unix_seconds: reader.number()?,
                expires_at_unix_seconds: reader.number()?,
                protocol_version: BoundaryProtocolVersion::try_new(u32::from_le_bytes(
                    reader.take()?,
                ))
                .map_err(|_| MalformedFrame)?,
                correlation_family: reader.text()?,
                correlation_token: reader.take()?,
                payload: reader.blob()?,
            };
            if claims.issued_at_unix_seconds > claims.expires_at_unix_seconds {
                return Err(MalformedFrame);
            }
            let envelope_bytes = reader.number()?;
            let source = super::source::decode(reader)?;
            QueryInboundRecord::Accepted(QueryInboundAcceptedMessage {
                operation,
                claims,
                envelope_bytes,
                source,
            })
        }
        6 => {
            let correlation = CanonicalDigestId::new(reader.take()?);
            let operation = reader.text()?;
            let audience = match reader.take::<1>()?[0] {
                0 => None,
                1 => Some(reader.text()?),
                _ => return Err(MalformedFrame),
            };
            QueryInboundRecord::TransportBinding(QueryTransportBinding {
                correlation,
                operation,
                audience,
                source: super::source::decode(reader)?,
            })
        }
        4 => QueryInboundRecord::Acknowledged {
            message: reader.message_key()?,
            completion_world: reader.number()?,
        },
        5 => QueryInboundRecord::Expired {
            message: reader.message_key()?,
        },
        _ => return Err(MalformedFrame),
    })
}

/// Exact physical grammar widths, without allocating an encoding before admission.
pub(in crate::domain_computation::application_aftermath::dispatch_journal) fn accepted_retained_bytes(
    operation: &str,
    claims: &WorthQueryInboundOccurrenceClaims,
    envelope_bytes: u64,
    native_branch: &str,
) -> Option<u64> {
    const VARIABLE_FIELDS: u64 = 7;
    const FIXED_RECORD_BYTES: u64 =
        1 + VARIABLE_FIELDS * 8 + 8 + 32 + 32 + 8 + 8 + 4 + 32 + 8 + 6 * 8 + 3 * 8;
    [
        operation.len(),
        claims.audience.len(),
        claims.protocol_identity.as_str().len(),
        claims.source_identity.len(),
        claims.correlation_family.len(),
        claims.payload.len(),
        native_branch.len(),
    ]
    .into_iter()
    .try_fold(
        FRAME_ENVELOPE_BYTES + FIXED_RECORD_BYTES,
        |bytes, length| bytes.checked_add(length as u64),
    )?
    .checked_add(envelope_bytes)
}
