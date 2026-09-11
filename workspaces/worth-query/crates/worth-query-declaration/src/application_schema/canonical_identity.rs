use worth_foundational::facade::{prepare_canonical_basis_sequence, CanonicalizationRuleVersion};

use super::canonical_basis::{
    ApplicationSchemaCanonicalBasis, ApplicationSchemaCanonicalBasisBudgetDenial,
    ApplicationSchemaCanonicalBasisWork, APPLICATION_SCHEMA_DOMAIN,
};
use super::{
    ApplicationSchemaContributionProvenance, ApplicationSchemaIdentity, ApplicationSchemaMember,
};

const LEGACY_RULE_VERSION: &str = "worth-query-application-schema-v11";
const REVISED_RULE_VERSION: &str = "worth-query-application-schema-v12";

pub(super) struct ApplicationSchemaCanonicalHeader<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub major: u32,
    pub minor: u32,
}

pub(super) fn canonical_identity(
    header: ApplicationSchemaCanonicalHeader<'_>,
    members: &[ApplicationSchemaMember],
    contributions: &[ApplicationSchemaContributionProvenance],
) -> ApplicationSchemaIdentity {
    canonical_identity_with_limits(header, members, contributions, u64::MAX, u64::MAX)
        .expect("ordinary schema construction uses an unbounded source observation")
        .0
}

pub(super) fn canonical_identity_with_limits(
    header: ApplicationSchemaCanonicalHeader<'_>,
    members: &[ApplicationSchemaMember],
    contributions: &[ApplicationSchemaContributionProvenance],
    maximum_source_bytes: u64,
    maximum_entries: u64,
) -> Result<
    (
        ApplicationSchemaIdentity,
        ApplicationSchemaCanonicalBasisWork,
    ),
    ApplicationSchemaCanonicalBasisBudgetDenial,
> {
    let revised = uses_revised_preimage(members, contributions);
    let contribution_entries = contribution_entry_capacity(contributions);
    let mut canonical = ApplicationSchemaCanonicalBasis::with_capacity_and_limits(
        members.len(),
        contribution_entries,
        maximum_source_bytes,
        maximum_entries,
    );
    canonical.text("header.owner", header.owner);
    canonical.text("header.name", header.name);
    canonical.u32("header.major", header.major);
    canonical.u32("header.minor", header.minor);
    if revised {
        append_contributions(&mut canonical, contributions);
    }
    canonical.usize("member-count", members.len());
    for (index, member) in members.iter().enumerate() {
        append_member(&mut canonical, index, member, revised);
        if canonical.is_denied() {
            break;
        }
    }
    let version = CanonicalizationRuleVersion::new(if revised {
        REVISED_RULE_VERSION
    } else {
        LEGACY_RULE_VERSION
    })
    .expect("the schema identity rule is valid");
    let (entries, work) = canonical.into_entries()?;
    let basis = prepare_canonical_basis_sequence(version, APPLICATION_SCHEMA_DOMAIN, entries)
        .into_result()
        .expect("schema identity loci are unique and typed");
    Ok((ApplicationSchemaIdentity::from_canonical_basis(basis), work))
}

fn contribution_entry_capacity(contributions: &[ApplicationSchemaContributionProvenance]) -> usize {
    contributions.iter().fold(1usize, |entries, contribution| {
        entries
            .saturating_add(2)
            .saturating_add(contribution.member_ordinals().len())
    })
}

fn append_contributions(
    canonical: &mut ApplicationSchemaCanonicalBasis,
    contributions: &[ApplicationSchemaContributionProvenance],
) {
    canonical.usize("contribution-count", contributions.len());
    for (contribution_index, contribution) in contributions.iter().enumerate() {
        canonical.text(
            format!("contribution[{contribution_index}].identity"),
            contribution.identity().as_str(),
        );
        canonical.usize(
            format!("contribution[{contribution_index}].member-count"),
            contribution.member_ordinals().len(),
        );
        for (member_index, ordinal) in contribution.member_ordinals().iter().enumerate() {
            canonical.u32(
                format!("contribution[{contribution_index}].member[{member_index}].ordinal"),
                *ordinal,
            );
        }
    }
}

fn uses_revised_preimage(
    members: &[ApplicationSchemaMember],
    contributions: &[ApplicationSchemaContributionProvenance],
) -> bool {
    !contributions.is_empty()
        || members.iter().any(|member| match member {
            ApplicationSchemaMember::Field { frame, .. } => frame.is_some(),
            ApplicationSchemaMember::Relation { integrity, .. } => {
                *integrity
                    != super::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            }
            _ => false,
        })
}

mod member;
use member::append_member;

#[cfg(test)]
#[path = "canonical_identity_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "canonical_identity_version_tests.rs"]
mod version_tests;
