//! Authority-free application-schema carriage and fresh declaration readmission.

use super::canonical_basis::ApplicationSchemaCanonicalBasisBudgetDenial;
use super::canonical_identity::{canonical_identity_with_limits, ApplicationSchemaCanonicalHeader};
use super::contribution::validate_canonical_contributions;
use super::identifier_validation::{validate_member_identifiers, validate_schema_header};
use super::member_closure::validate_member_closure;
use super::operation_contract_cardinality::validate_operation_contract_cardinality;
use super::{
    ApplicationSchemaContributionProvenance, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationSchemaRecord {
    owner: String,
    name: String,
    major: u32,
    minor: u32,
    members: Vec<ApplicationSchemaMember>,
    contributions: Vec<ApplicationSchemaContributionProvenance>,
}

impl WorthQueryPortableApplicationSchemaRecord {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationSchemaParts) -> Self {
        Self {
            owner: parts.owner,
            name: parts.name,
            major: parts.major,
            minor: parts.minor,
            members: parts.members.into_iter().map(without_live_recipe).collect(),
            contributions: parts.contributions,
        }
    }

    pub fn project(source: &ErasedApplicationSchemaDeclaration) -> Self {
        Self {
            owner: source.owner().to_owned(),
            name: source.name().to_owned(),
            major: source.major(),
            minor: source.minor(),
            members: source
                .members()
                .iter()
                .cloned()
                .map(without_live_recipe)
                .collect(),
            contributions: source.contributions().to_vec(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn major(&self) -> u32 {
        self.major
    }

    pub const fn minor(&self) -> u32 {
        self.minor
    }

    pub fn members(&self) -> &[ApplicationSchemaMember] {
        &self.members
    }

    pub fn contributions(&self) -> &[ApplicationSchemaContributionProvenance] {
        &self.contributions
    }

    pub fn into_parts(self) -> WorthQueryPortableApplicationSchemaParts {
        WorthQueryPortableApplicationSchemaParts {
            owner: self.owner,
            name: self.name,
            major: self.major,
            minor: self.minor,
            members: self.members,
            contributions: self.contributions,
        }
    }
}

fn without_live_recipe(member: ApplicationSchemaMember) -> ApplicationSchemaMember {
    match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => {
            ApplicationSchemaMember::ApplicationCapability {
                contract: crate::application_capability::ErasedApplicationCapabilityContract::from_untrusted_parts(
                    contract.into_parts(),
                ),
            }
        }
        member => member,
    }
}

pub struct WorthQueryPortableApplicationSchemaParts {
    pub owner: String,
    pub name: String,
    pub major: u32,
    pub minor: u32,
    pub members: Vec<ApplicationSchemaMember>,
    pub contributions: Vec<ApplicationSchemaContributionProvenance>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationSchemaReadmissionWork {
    source_bytes: u64,
    canonical_entries: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableApplicationSchemaWorkObservationDenial {
    SourceByteBudgetExceeded { observed: u64, maximum: u64 },
    NestedEntryBudgetExceeded { observed: u64, maximum: u64 },
}

impl WorthQueryPortableApplicationSchemaReadmissionWork {
    pub const fn source_bytes(self) -> u64 {
        self.source_bytes
    }

    pub const fn canonical_entries(self) -> u64 {
        self.canonical_entries
    }
}

pub fn validate_portable_application_schema_freshly(
    record: WorthQueryPortableApplicationSchemaRecord,
) -> Result<ErasedApplicationSchemaDeclaration, Denial> {
    validate_portable_application_schema_freshly_with_work(record, u64::MAX, u64::MAX)
        .map(|(declaration, _work)| declaration)
}

pub fn validate_portable_application_schema_freshly_with_work(
    record: WorthQueryPortableApplicationSchemaRecord,
    maximum_source_bytes: u64,
    maximum_canonical_entries: u64,
) -> Result<
    (
        ErasedApplicationSchemaDeclaration,
        WorthQueryPortableApplicationSchemaReadmissionWork,
    ),
    Denial,
> {
    let (identity, work) = canonical_identity_with_limits(
        ApplicationSchemaCanonicalHeader {
            owner: &record.owner,
            name: &record.name,
            major: record.major,
            minor: record.minor,
        },
        &record.members,
        &record.contributions,
        maximum_source_bytes,
        maximum_canonical_entries,
    )
    .map_err(map_work_denial)?;
    validate_schema_header(&record.owner, &record.name)?;
    validate_member_identifiers(&record.members)?;
    validate_portable_application_queries(&record.members)?;
    super::member_identity_uniqueness::validate_member_identity_uniqueness(&record.members)?;
    validate_operation_contract_cardinality(&record.members)?;
    if !record.members.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(Denial::InvalidCanonicalOrdering);
    }
    validate_canonical_contributions(&record.contributions, record.members.len())?;
    validate_member_closure(&record.members)?;
    Ok((
        ErasedApplicationSchemaDeclaration::from_fresh_parts(
            record.owner,
            record.name,
            record.major,
            record.minor,
            identity,
            record.members,
            record.contributions,
        ),
        WorthQueryPortableApplicationSchemaReadmissionWork {
            source_bytes: work.source_bytes,
            canonical_entries: work.entries,
        },
    ))
}

fn validate_portable_application_queries(
    members: &[ApplicationSchemaMember],
) -> Result<(), Denial> {
    members.iter().try_for_each(|member| {
        let ApplicationSchemaMember::ApplicationQuery { definition } = member else {
            return Ok(());
        };
        crate::application_query::validate_portable_application_query_freshly(definition.parts())
            .map_err(|_| Denial::InvalidApplicationQuery)
    })
}

pub fn observe_portable_application_schema_reconstruction_work(
    record: &WorthQueryPortableApplicationSchemaRecord,
    maximum_source_bytes: u64,
    maximum_canonical_entries: u64,
) -> Result<
    WorthQueryPortableApplicationSchemaReadmissionWork,
    WorthQueryPortableApplicationSchemaWorkObservationDenial,
> {
    let (_, work) = canonical_identity_with_limits(
        ApplicationSchemaCanonicalHeader {
            owner: &record.owner,
            name: &record.name,
            major: record.major,
            minor: record.minor,
        },
        &record.members,
        &record.contributions,
        maximum_source_bytes,
        maximum_canonical_entries,
    )
    .map_err(|denial| match denial {
        ApplicationSchemaCanonicalBasisBudgetDenial::SourceBytes { observed, maximum } => {
            WorthQueryPortableApplicationSchemaWorkObservationDenial::SourceByteBudgetExceeded {
                observed,
                maximum,
            }
        }
        ApplicationSchemaCanonicalBasisBudgetDenial::Entries { observed, maximum } => {
            WorthQueryPortableApplicationSchemaWorkObservationDenial::NestedEntryBudgetExceeded {
                observed,
                maximum,
            }
        }
    })?;
    Ok(WorthQueryPortableApplicationSchemaReadmissionWork {
        source_bytes: work.source_bytes,
        canonical_entries: work.entries,
    })
}

fn map_work_denial(denial: ApplicationSchemaCanonicalBasisBudgetDenial) -> Denial {
    match denial {
        ApplicationSchemaCanonicalBasisBudgetDenial::SourceBytes { observed, maximum } => {
            Denial::PortableCanonicalSourceBytesBudgetExceeded { observed, maximum }
        }
        ApplicationSchemaCanonicalBasisBudgetDenial::Entries { observed, maximum } => {
            Denial::PortableCanonicalEntryBudgetExceeded { observed, maximum }
        }
    }
}
