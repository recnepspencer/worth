use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::application_query::WorthQueryInstalledApplicationQueryIdentity;
use crate::authority_cryptography::{
    AuthoritySeal, AuthoritySealDomain, AuthorityTranscript, PackageAuthorityKey,
};
use crate::graph_obligation::WorthQueryInstalledGraphObligationSetIdentity;

pub(super) fn derive_installed_query_authority_seal(
    key: &PackageAuthorityKey,
    binding: &ApplicationSchemaBindingIdentity,
    query_identity: &WorthQueryInstalledApplicationQueryIdentity,
    obligations: &WorthQueryInstalledGraphObligationSetIdentity,
) -> AuthoritySeal {
    authority_transcript(key, binding, query_identity, obligations).finish()
}

pub(super) fn verify_installed_query_authority_seal(
    seal: &AuthoritySeal,
    key: &PackageAuthorityKey,
    binding: &ApplicationSchemaBindingIdentity,
    query_identity: &WorthQueryInstalledApplicationQueryIdentity,
    obligations: &WorthQueryInstalledGraphObligationSetIdentity,
) -> bool {
    authority_transcript(key, binding, query_identity, obligations).verifies(seal)
}

fn authority_transcript(
    key: &PackageAuthorityKey,
    binding: &ApplicationSchemaBindingIdentity,
    query_identity: &WorthQueryInstalledApplicationQueryIdentity,
    obligations: &WorthQueryInstalledGraphObligationSetIdentity,
) -> AuthorityTranscript {
    let mut transcript =
        AuthorityTranscript::new(key, AuthoritySealDomain::InstalledApplicationQuery);
    transcript.bytes("package", binding.package_identity().bytes());
    transcript.bytes("schema", binding.schema_identity().bytes());
    transcript.bytes("query", query_identity.as_bytes());
    transcript.bytes("graph-obligations", obligations.bytes());
    transcript
}

/// Fixed initialized Work for validating one installed Query identity and
/// its authority seal. The transcript has four 32-byte fields; framing,
/// domain separation, and HMAC compression are part of the same check.
pub(crate) fn installed_query_validation_work_bound() -> Option<u64> {
    let domain = AuthoritySealDomain::InstalledApplicationQuery.label();
    let framed = [
        (b"domain".len(), domain.len(), false),
        (b"package".len(), 32, true),
        (b"schema".len(), 32, true),
        (b"query".len(), 32, true),
        (b"graph-obligations".len(), 32, true),
    ];
    let transcript = framed
        .into_iter()
        .try_fold(0usize, |sum, (tag, value, typed)| {
            let kind = if typed { b"bytes".len() + 8 } else { 0 };
            sum.checked_add(tag + 8)?
                .checked_add(kind)?
                .checked_add(value + 8)
        })?;
    let inner_blocks = transcript.checked_add(9)?.div_ceil(64);
    let hash_blocks = inner_blocks.checked_add(3)?;
    let work = transcript
        .checked_add(128)?
        .checked_add(32)?
        .checked_add(hash_blocks)?
        .checked_add(4)?;
    u64::try_from(work).ok()
}
