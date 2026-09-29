use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};

use super::{BankProposalDenial, CanonicalProposalPayload};

const KEY_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.idempotency-key-identity");
const KEY_RULE_VERSION: &str = "worth-bank-idempotency-key-identity-v2";
const INTENT_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.idempotency-intent");
const INTENT_RULE_VERSION: &str = "worth-bank-idempotency-intent-v2";

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize)]
pub struct BankIdempotencyKey(String);

impl BankIdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, BankProposalDenial> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(BankProposalDenial::InvalidIdempotencyKey);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankOperationScopeSchemaBinding {
    runtime_ordinal: u64,
    generation: u64,
    package_identity: CanonicalDigestId,
    schema_identity: CanonicalDigestId,
}

impl BankOperationScopeSchemaBinding {
    pub const fn new(
        runtime_ordinal: u64,
        generation: u64,
        package_identity: [u8; 32],
        schema_identity: [u8; 32],
    ) -> Self {
        Self {
            runtime_ordinal,
            generation,
            package_identity: CanonicalDigestId::new(package_identity),
            schema_identity: CanonicalDigestId::new(schema_identity),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankOperationScopeEntityBinding {
    partition_id: u32,
    local_slot: u64,
    generation: u32,
}

impl BankOperationScopeEntityBinding {
    pub const fn new(partition_id: u32, local_slot: u64, generation: u32) -> Self {
        Self {
            partition_id,
            local_slot,
            generation,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BankOperationScopeBinding {
    runtime_authority: u64,
    schema: BankOperationScopeSchemaBinding,
    operation_authority_identity: String,
    principal: BankOperationScopeEntityBinding,
    scope: BankOperationScopeEntityBinding,
}

impl BankOperationScopeBinding {
    /// Retains Query's descriptive operation-scope components for the Bank
    /// idempotency seam. This value carries no Query execution authority.
    pub fn new(
        runtime_authority: u64,
        schema: BankOperationScopeSchemaBinding,
        operation_authority_identity: impl Into<String>,
        principal: BankOperationScopeEntityBinding,
        scope: BankOperationScopeEntityBinding,
    ) -> Self {
        Self {
            runtime_authority,
            schema,
            operation_authority_identity: operation_authority_identity.into(),
            principal,
            scope,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankIdempotencyIntent(CanonicalDigestId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankIdempotencyKeyIdentity(CanonicalDigestId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankIdempotencyClaim {
    key: BankIdempotencyKeyIdentity,
    intent: BankIdempotencyIntent,
}

impl BankIdempotencyClaim {
    /// Retains an application runtime's descriptive idempotency identities for
    /// deterministic domain-created IDs. This carries no Query authority and
    /// does not replace the runtime's own retry or intent-drift decision.
    pub const fn from_application_binding(
        key_identity: [u8; 32],
        intent_identity: [u8; 32],
    ) -> Self {
        Self {
            key: BankIdempotencyKeyIdentity(CanonicalDigestId::new(key_identity)),
            intent: BankIdempotencyIntent(CanonicalDigestId::new(intent_identity)),
        }
    }

    /// Deterministic journal identity produced by this admitted operation.
    pub const fn journal_identity(self, ordinal: u32) -> crate::model::JournalEntryId {
        crate::model::JournalEntryId::from_operation(*self.key.0.bytes(), ordinal)
    }

    pub(crate) fn derive(
        binding: BankOperationScopeBinding,
        key: &BankIdempotencyKey,
        payload: CanonicalProposalPayload,
    ) -> Self {
        let operation = payload.operation();
        let payload_identity = payload.derive_identity();
        let key = BankIdempotencyKeyIdentity(derive_identity(
            KEY_DOMAIN,
            KEY_RULE_VERSION,
            [
                unsigned_entry(KEY_DOMAIN, "runtime-authority", binding.runtime_authority),
                unsigned_entry(KEY_DOMAIN, "schema-runtime", binding.schema.runtime_ordinal),
                unsigned_entry(KEY_DOMAIN, "schema-generation", binding.schema.generation),
                digest_entry(
                    KEY_DOMAIN,
                    "schema-package",
                    binding.schema.package_identity,
                ),
                digest_entry(
                    KEY_DOMAIN,
                    "schema-identity",
                    binding.schema.schema_identity,
                ),
                text_entry(
                    KEY_DOMAIN,
                    "operation-authority",
                    &binding.operation_authority_identity,
                ),
                entity_entry(KEY_DOMAIN, "principal", binding.principal),
                entity_entry(KEY_DOMAIN, "scope", binding.scope),
                text_entry(KEY_DOMAIN, "operation", operation),
                text_entry(KEY_DOMAIN, "client-key", key.as_str()),
            ],
        ));
        Self {
            key,
            intent: BankIdempotencyIntent(derive_identity(
                INTENT_DOMAIN,
                INTENT_RULE_VERSION,
                [
                    digest_entry(INTENT_DOMAIN, "key", key.0),
                    digest_entry(INTENT_DOMAIN, "proposal", payload_identity),
                ],
            )),
        }
    }

    pub const fn key(self) -> BankIdempotencyKeyIdentity {
        self.key
    }

    pub const fn intent(self) -> BankIdempotencyIntent {
        self.intent
    }
}

impl BankIdempotencyIntent {
    pub const fn bytes(self) -> [u8; 32] {
        *self.0.bytes()
    }

    pub(crate) fn canonical_text(self) -> String {
        canonical_hex(*self.0.bytes())
    }

    pub(crate) fn from_canonical_text(value: &str) -> Option<Self> {
        decode_hex(value).map(CanonicalDigestId::new).map(Self)
    }
}

impl BankIdempotencyKeyIdentity {
    pub const fn bytes(self) -> [u8; 32] {
        *self.0.bytes()
    }

    pub(crate) fn canonical_text(self) -> String {
        canonical_hex(*self.0.bytes())
    }

    pub(crate) fn from_canonical_text(value: &str) -> Option<Self> {
        decode_hex(value).map(CanonicalDigestId::new).map(Self)
    }
}

fn derive_identity<const N: usize>(
    domain: CanonicalBasisDomain,
    rule_version: &'static str,
    entries: [CanonicalBasisEntry; N],
) -> CanonicalDigestId {
    let version = CanonicalizationRuleVersion::new(rule_version)
        .expect("the fixed bank identity rule is valid");
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .expect("bank idempotency identity fields have unique typed loci");
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .expect("SHA-256 admits the typed bank idempotency basis");
    CanonicalDigestId::new(*canonicalization().digest().derive(ready).value().bytes())
}

fn text_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: &str,
) -> CanonicalBasisEntry {
    entry(
        domain,
        locus,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn digest_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: CanonicalDigestId,
) -> CanonicalBasisEntry {
    entry(domain, locus, CanonicalBasisValue::BytesDigest(value))
}

fn unsigned_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: u64,
) -> CanonicalBasisEntry {
    entry(
        domain,
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn entity_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: BankOperationScopeEntityBinding,
) -> CanonicalBasisEntry {
    entry(
        domain,
        locus,
        CanonicalBasisValue::EntityRef {
            partition_id: value.partition_id,
            local_slot: value.local_slot,
            generation: value.generation,
        },
    )
}

fn entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}

fn canonical_hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(64);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn decode_hex(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        decoded[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(decoded)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
