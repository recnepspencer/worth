use worth_foundational::facade::{
    canonicalization, CanonicalBasisReadyArtifact, CanonicalDigestAlgorithmId, CanonicalDigestId,
};

use super::ApplicationValueMeaningDenial;
use crate::application_schema::{
    canonical_identity::canonical_field_meaning, ApplicationFieldPresence,
    ApplicationSchemaIdentity, ApplicationSchemaMember,
};

/// Complete declared field semantics, produced by the ordinary schema emitter.
/// A digest observes this description; it grants no runtime authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFieldValueMeaning {
    member: ApplicationSchemaMember,
    identity: ApplicationSchemaIdentity,
    digest: CanonicalDigestId,
}

impl ApplicationFieldValueMeaning {
    pub fn from_member(
        member: &ApplicationSchemaMember,
    ) -> Result<Option<Self>, ApplicationValueMeaningDenial> {
        if !matches!(member, ApplicationSchemaMember::Field { .. }) {
            return Ok(None);
        }
        let identity = canonical_field_meaning(member);
        let ready = canonicalization()
            .digest()
            .for_sequence(
                identity.canonical_basis().clone(),
                CanonicalDigestAlgorithmId::sha256(),
            )
            .into_result()
            .map_err(ApplicationValueMeaningDenial::Canonical)?;
        let derived = canonicalization().digest().derive(ready);
        Ok(Some(Self {
            member: member.clone(),
            identity,
            digest: CanonicalDigestId::new(*derived.value().bytes()),
        }))
    }

    pub fn member(&self) -> &ApplicationSchemaMember {
        &self.member
    }
    pub fn name(&self) -> &str {
        let ApplicationSchemaMember::Field { field, .. } = &self.member else {
            unreachable!()
        };
        field
    }
    pub fn presence(&self) -> ApplicationFieldPresence {
        let ApplicationSchemaMember::Field { presence, .. } = &self.member else {
            unreachable!()
        };
        *presence
    }
    pub fn canonical_basis(&self) -> &CanonicalBasisReadyArtifact {
        self.identity.canonical_basis()
    }
    pub const fn semantic_digest(&self) -> CanonicalDigestId {
        self.digest
    }
    /// Legacy scalar owners remain unclassified at the physical codec boundary.
    pub fn codec_digest(&self) -> Option<CanonicalDigestId> {
        matches!(
            &self.member,
            ApplicationSchemaMember::Field {
                canonical: Some(_),
                ..
            }
        )
        .then_some(self.digest)
    }
}
