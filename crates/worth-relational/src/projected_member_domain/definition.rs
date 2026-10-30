//! Portable domain meaning and distinct registry-issued binding handles.
use crate::identity::data::KindId;
use crate::schema::data::{SchemaId, SchemaVersionId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ProjectedMemberDomainIdentity {
    pub schema: SchemaId,
    pub version: SchemaVersionId,
    pub name: String,
}
impl ProjectedMemberDomainIdentity {
    pub(crate) fn name_bytes(&self) -> Option<usize> {
        self.schema.0.len().checked_add(self.name.len())
    }
    pub(crate) fn owned_bytes(&self) -> usize {
        self.schema
            .0
            .capacity()
            .saturating_add(self.name.capacity())
    }
}

/// Cases are a nonempty, strictly increasing semantic roster. Readmission
/// rejects noncanonical input rather than silently normalizing it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ProjectedMemberDomainParts {
    pub identity: ProjectedMemberDomainIdentity,
    pub carrier: KindId,
    pub cases: Vec<String>,
}
impl ProjectedMemberDomainParts {
    pub(crate) fn owned_bytes(&self) -> usize {
        self.identity
            .owned_bytes()
            .saturating_add(self.cases.capacity().saturating_mul(size_of::<String>()))
            .saturating_add(self.cases.iter().map(String::capacity).sum::<usize>())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RegisteredProjectedMemberDomain {
    pub(crate) parts: ProjectedMemberDomainParts,
}

/// A domain binding issued by one registry allocation, never reconstructed
/// from equal portable meaning.
#[derive(Clone, Debug)]
pub struct ProjectedMemberDomainHandle {
    pub(crate) entry: Arc<RegisteredProjectedMemberDomain>,
}
impl PartialEq for ProjectedMemberDomainHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
}
impl Eq for ProjectedMemberDomainHandle {}
impl ProjectedMemberDomainHandle {
    pub fn parts(&self) -> &ProjectedMemberDomainParts {
        &self.entry.parts
    }

    pub fn case(&self, name: &str) -> Option<ProjectedMemberCaseHandle> {
        self.parts()
            .cases
            .binary_search_by(|case| case.as_str().cmp(name))
            .ok()
            .map(|index| ProjectedMemberCaseHandle {
                domain: self.clone(),
                index,
            })
    }
}

/// Retains its actual registered domain; the index is private routing only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMemberCaseHandle {
    pub(crate) domain: ProjectedMemberDomainHandle,
    pub(crate) index: usize,
}
impl ProjectedMemberCaseHandle {
    pub fn domain(&self) -> &ProjectedMemberDomainHandle {
        &self.domain
    }
    pub fn name(&self) -> &str {
        &self.domain.parts().cases[self.index]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedMemberDomainDenial {
    InvalidIdentity,
    InvalidCarrier(KindId),
    EmptyCases,
    NoncanonicalCases,
    DuplicateDomain(ProjectedMemberDomainIdentity),
    ForeignDomain,
    ForeignCase,
}
