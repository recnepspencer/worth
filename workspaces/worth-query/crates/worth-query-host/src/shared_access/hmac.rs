//! Pure RFC 2104 HMAC-SHA256, constant-time tag comparison, and canonical token codec.

use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::{
    application_schema::ApplicationReadableScalarValueBinding,
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    },
};

use super::capped_writer::RemoteByteWriter;
use super::denial::{AuthenticationRefusalReason, SharedAccessDenial};
use super::framing::{BinaryReader, BinaryWriter};

pub(super) const TOKEN_MAC_DOMAIN: &[u8] = b"worth.query.shared_access.token.v1\0";
pub(super) const MAX_KEY_ID_BYTES: usize = 64;
pub(super) const MAX_TOKEN_BYTES: usize = 2048;
/// A token principal travels as its identity binding's length-framed index
/// encoding, so decoding reuses the identity owner's codec.
const MAX_PRINCIPAL_BYTES: usize = 600;

#[derive(Clone, Eq, PartialEq)]
pub(super) struct TokenBody {
    pub(super) key_id: String,
    pub(super) home_identity: [u8; 16],
    pub(super) pinned_incarnation: Option<u64>,
    pub(super) principal: WorthQueryExternalPrincipalIdentity,
    pub(super) issued_at_unix_secs: u64,
    pub(super) expires_at_unix_secs: u64,
    pub(super) nonce: [u8; 16],
}

pub(super) fn encode_token_body(writer: &mut BinaryWriter, body: &TokenBody) {
    writer.write_str(&body.key_id);
    writer.write_fixed(&body.home_identity);
    match body.pinned_incarnation {
        Some(inc) => {
            writer.write_u8(1);
            writer.write_u64(inc);
        }
        None => writer.write_u8(0),
    }
    writer.write_str(&body.principal.portable_text());
    writer.write_u64(body.issued_at_unix_secs);
    writer.write_u64(body.expires_at_unix_secs);
    writer.write_fixed(&body.nonce);
}

pub(super) fn encode_token_body_capped(
    writer: &mut RemoteByteWriter,
    body: &TokenBody,
) -> Result<(), SharedAccessDenial> {
    writer.write_str(&body.key_id)?;
    writer.write_fixed(&body.home_identity)?;
    match body.pinned_incarnation {
        Some(inc) => {
            writer.write_u8(1)?;
            writer.write_u64(inc)?;
        }
        None => writer.write_u8(0)?,
    }
    writer.write_str(&body.principal.portable_text())?;
    writer.write_u64(body.issued_at_unix_secs)?;
    writer.write_u64(body.expires_at_unix_secs)?;
    writer.write_fixed(&body.nonce)
}

pub(super) fn decode_token_parts(
    reader: &mut BinaryReader<'_>,
) -> Result<(TokenBody, [u8; 32]), SharedAccessDenial> {
    let key_id = reader.read_str_bounded(MAX_KEY_ID_BYTES)?;
    let home_identity = reader.read_fixed::<16>()?;
    let pinned_incarnation = match reader.read_u8()? {
        0 => None,
        1 => Some(reader.read_u64()?),
        _ => {
            return Err(SharedAccessDenial::AuthenticationRefused(
                AuthenticationRefusalReason::MalformedToken,
            ));
        }
    };
    let principal = decode_principal(&reader.read_str_bounded(MAX_PRINCIPAL_BYTES)?)?;
    let issued_at_unix_secs = reader.read_u64()?;
    let expires_at_unix_secs = reader.read_u64()?;
    let nonce = reader.read_fixed::<16>()?;
    let mac = reader.read_fixed::<32>()?;
    Ok((
        TokenBody {
            key_id,
            home_identity,
            pinned_incarnation,
            principal,
            issued_at_unix_secs,
            expires_at_unix_secs,
            nonce,
        },
        mac,
    ))
}

pub(super) fn compute_token_mac(secret: &[u8; 32], body: &TokenBody) -> [u8; 32] {
    let mut writer = BinaryWriter::with_capacity(256);
    writer.write_fixed(TOKEN_MAC_DOMAIN);
    encode_token_body(&mut writer, body);
    hmac_sha256(secret, &writer.into_bytes())
}

/// Pure RFC 2104 HMAC-SHA256 using workspace `sha2`.
pub(super) fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut block = [0_u8; 64];
    if key.len() > 64 {
        let hashed = Sha256::digest(key);
        block[..32].copy_from_slice(&hashed);
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36_u8; 64];
    let mut opad = [0x5c_u8; 64];
    for idx in 0..64 {
        ipad[idx] ^= block[idx];
        opad[idx] ^= block[idx];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    outer.finalize().into()
}

/// Constant-time 32-byte equality check (no early exit).
pub(super) fn constant_time_eq_32(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut diff = 0_u8;
    for idx in 0..32 {
        diff |= left[idx] ^ right[idx];
    }
    std::hint::black_box(diff) == 0
}

fn malformed_token() -> SharedAccessDenial {
    SharedAccessDenial::AuthenticationRefused(AuthenticationRefusalReason::MalformedToken)
}

fn decode_principal(
    encoded: &str,
) -> Result<WorthQueryExternalPrincipalIdentity, SharedAccessDenial> {
    WorthQueryExternalPrincipalIdentityBinding::decode(&AspectValue::String(InternedString::from(
        encoded,
    )))
    .map_err(|_| malformed_token())
}
