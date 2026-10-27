use worth_foundational::facade::ScalarAspectType;

use crate::application_schema::{ApplicationCanonicalValueContract, ApplicationFieldPresence};

/// Portable field meaning within an explicitly relation-owned aspect.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationFieldDefinition {
    pub field: String,
    pub presence: ApplicationFieldPresence,
    pub scalar_family: ScalarAspectType,
    pub value_type: String,
    pub canonical: Option<ApplicationCanonicalValueContract>,
    pub unit: Option<String>,
    pub frame: Option<String>,
    pub writable: bool,
}

impl ApplicationRelationFieldDefinition {
    pub fn codec_digest(
        &self,
        relation: &str,
        aspect: &str,
    ) -> Option<worth_foundational::facade::CanonicalDigestId> {
        self.canonical.as_ref()?;
        let meaning = super::super::canonical_identity::canonical_relation_field_meaning(
            relation, aspect, self,
        );
        let ready = worth_foundational::facade::canonicalization()
            .digest()
            .for_sequence(
                meaning.canonical_basis().clone(),
                worth_foundational::facade::CanonicalDigestAlgorithmId::sha256(),
            )
            .into_result()
            .expect("relation field codec meaning is canonical");
        let derived = worth_foundational::facade::canonicalization()
            .digest()
            .derive(ready);
        Some(worth_foundational::facade::CanonicalDigestId::new(
            *derived.value().bytes(),
        ))
    }
}
