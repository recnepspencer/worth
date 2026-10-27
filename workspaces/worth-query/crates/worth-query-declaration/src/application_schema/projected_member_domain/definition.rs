//! Versioned carrier/case meaning, independent of live graph authority.
use super::{ApplicationProjectedMemberCaseRef, ApplicationProjectedMemberDomainRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationProjectedMemberDomainDefinitionDenial {
    UnsupportedVersion { observed: u16 },
    InvalidIdentifier,
    EmptyCases,
    DuplicateCase,
    MissingCarrier,
    DuplicateDomain,
    EntityNameCollision,
    ForeignCase,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableProjectedMemberDomainParts {
    pub semantic_version: u16,
    pub domain: String,
    pub carrier: String,
    pub cases: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ErasedApplicationProjectedMemberDomainDefinition {
    parts: WorthQueryPortableProjectedMemberDomainParts,
}
impl ErasedApplicationProjectedMemberDomainDefinition {
    pub const CURRENT_VERSION: u16 = 1;

    /// Normalize semantic case order; duplicates remain visible to validation.
    pub fn from_untrusted_parts(mut parts: WorthQueryPortableProjectedMemberDomainParts) -> Self {
        parts.cases.sort();
        Self { parts }
    }
    pub fn parts(&self) -> &WorthQueryPortableProjectedMemberDomainParts {
        &self.parts
    }
    pub fn validate(&self) -> Result<(), ApplicationProjectedMemberDomainDefinitionDenial> {
        super::validation::validate_definition(&self.parts)
    }
}

pub struct ApplicationProjectedMemberDomainDefinition<Schema, Domain, Carrier> {
    reference: ApplicationProjectedMemberDomainRef<Schema, Domain, Carrier>,
    cases: Vec<String>,
}
impl<S, D, C> ApplicationProjectedMemberDomainDefinition<S, D, C> {
    pub fn new(reference: ApplicationProjectedMemberDomainRef<S, D, C>) -> Self {
        Self {
            reference,
            cases: Vec::new(),
        }
    }
    pub fn case<Case>(
        mut self,
        case: ApplicationProjectedMemberCaseRef<S, D, Case>,
    ) -> Result<Self, ApplicationProjectedMemberDomainDefinitionDenial> {
        if case.domain() != self.reference.name() {
            return Err(ApplicationProjectedMemberDomainDefinitionDenial::ForeignCase);
        }
        self.cases.push(case.name().to_owned());
        Ok(self)
    }
    pub(super) fn into_erased(self) -> ErasedApplicationProjectedMemberDomainDefinition {
        ErasedApplicationProjectedMemberDomainDefinition::from_untrusted_parts(
            WorthQueryPortableProjectedMemberDomainParts {
                semantic_version: ErasedApplicationProjectedMemberDomainDefinition::CURRENT_VERSION,
                domain: self.reference.name().into(),
                carrier: self.reference.carrier().into(),
                cases: self.cases,
            },
        )
    }
}
