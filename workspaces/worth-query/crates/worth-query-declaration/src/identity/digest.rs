use super::digest_hash::digest_hash_parts;
use crate::archive_decode_authority::ArchiveDecodeAuthority;
use std::fmt::Write;
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDerivedDigest,
    CanonicalDigestAlgorithmId, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalizationRuleVersion,
};

macro_rules! declaration_digest {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(String);

        impl $name {
            /// Retains one descriptive digest claim decoded from untrusted
            /// storage without minting canonicalization authority.
            pub fn from_untrusted(_: ArchiveDecodeAuthority, value: String) -> Self {
                Self(value)
            }

            pub fn from_parts(parts: &[String]) -> Self {
                Self(digest_hash_parts(parts))
            }

            /// Carries a digest already derived through Foundational's admitted
            /// canonical slot. This remains descriptive identity, not authority.
            pub fn from_canonical_digest(digest: &CanonicalDerivedDigest) -> Self {
                let mut encoded = String::with_capacity(64);
                for byte in digest.value().bytes() {
                    write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
                }
                Self(encoded)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

declaration_digest!(CanonicalQueryDigest);
declaration_digest!(CanonicalResultShapeDigest);
declaration_digest!(ValidatedQueryDigest);
declaration_digest!(ValidatedResultShapeDigest);
declaration_digest!(CollectionPlanDigest);
declaration_digest!(BindingFulfillmentDigest);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SchemaBasisDigest(CanonicalDigestId);

impl SchemaBasisDigest {
    pub fn from_parts(parts: &[String]) -> Self {
        let domain = CanonicalBasisDomain::Future("worth-query.mature-schema-basis");
        let entries = parts.iter().enumerate().map(|(index, part)| {
            CanonicalBasisEntry::new(
                domain,
                CanonicalBasisLocus::Named(format!("part[{index}]").into()),
                CanonicalBasisEntryKind::Field,
                CanonicalBasisValue::ExactText(part.clone().into()),
            )
        });
        let basis = prepare_canonical_basis_sequence(
            CanonicalizationRuleVersion::new("worth-query-mature-schema-basis-v2")
                .expect("the fixed mature schema-basis rule is valid"),
            domain,
            entries,
        )
        .into_result()
        .expect("mature schema meaning has a nonempty basis");
        let budget = CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
            .expect("the mature schema-basis canonical budget is nonzero");
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
            .into_result()
            .expect("declared mature schemas fit the installed canonical-work budget");
        Self(CanonicalDigestId::new(
            *canonicalization().digest().derive(ready).value().bytes(),
        ))
    }

    pub fn from_canonical_digest(digest: &CanonicalDerivedDigest) -> Self {
        Self(CanonicalDigestId::new(*digest.value().bytes()))
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }
}

pub use super::digest_hash::hash_parts;
