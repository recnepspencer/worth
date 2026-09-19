use super::{ApplicationSchemaContributionIdentity, ApplicationSchemaContributionProvenance};
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember,
};

#[derive(Clone, Debug)]
pub(in crate::application_schema) struct AuthoredApplicationSchemaContribution {
    identity: ApplicationSchemaContributionIdentity,
    members: Vec<ApplicationSchemaMember>,
}

impl AuthoredApplicationSchemaContribution {
    pub(in crate::application_schema) fn new(
        identity: ApplicationSchemaContributionIdentity,
        members: Vec<ApplicationSchemaMember>,
    ) -> Self {
        Self { identity, members }
    }
}

pub(in crate::application_schema) fn lower_authored_contributions(
    authored: Vec<AuthoredApplicationSchemaContribution>,
    canonical_members: &[ApplicationSchemaMember],
) -> Result<Vec<ApplicationSchemaContributionProvenance>, Denial> {
    if authored.is_empty() {
        return Ok(Vec::new());
    }
    let mut contributions = Vec::with_capacity(authored.len());
    for contribution in authored {
        if !contribution.identity.is_valid() {
            return Err(Denial::InvalidContributionIdentity);
        }
        if contribution.members.is_empty() {
            return Err(Denial::EmptyContributionClosure);
        }
        let mut member_ordinals = contribution
            .members
            .iter()
            .map(|member| canonical_member_ordinal(canonical_members, member))
            .collect::<Result<Vec<_>, _>>()?;
        member_ordinals.sort_unstable();
        contributions.push(
            ApplicationSchemaContributionProvenance::from_canonical_parts(
                contribution.identity,
                member_ordinals,
            ),
        );
    }
    contributions.sort_by(|left, right| left.identity().cmp(right.identity()));
    validate_canonical_contributions(&contributions, canonical_members.len())?;
    Ok(contributions)
}

pub(in crate::application_schema) fn validate_canonical_contributions(
    contributions: &[ApplicationSchemaContributionProvenance],
    member_count: usize,
) -> Result<(), Denial> {
    if contributions.is_empty() {
        return Ok(());
    }
    for pair in contributions.windows(2) {
        match pair[0].identity().cmp(pair[1].identity()) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => return Err(Denial::DuplicateContributionIdentity),
            std::cmp::Ordering::Greater => return Err(Denial::InvalidContributionOrdering),
        }
    }
    let mut ownership = vec![false; member_count];
    for contribution in contributions {
        if !contribution.identity().is_valid() {
            return Err(Denial::InvalidContributionIdentity);
        }
        let ordinals = contribution.member_ordinals();
        if ordinals.is_empty() {
            return Err(Denial::EmptyContributionClosure);
        }
        if !ordinals.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(Denial::InvalidContributionOrdering);
        }
        for ordinal in ordinals {
            let index =
                usize::try_from(*ordinal).map_err(|_| Denial::InvalidContributionMemberOrdinal)?;
            let owned = ownership
                .get_mut(index)
                .ok_or(Denial::InvalidContributionMemberOrdinal)?;
            if *owned {
                return Err(Denial::OverlappingContributionClosure);
            }
            *owned = true;
        }
    }
    if ownership.iter().any(|owned| !owned) {
        return Err(Denial::IncompleteContributionClosure);
    }
    Ok(())
}

fn canonical_member_ordinal(
    canonical_members: &[ApplicationSchemaMember],
    member: &ApplicationSchemaMember,
) -> Result<u32, Denial> {
    let index = canonical_members
        .binary_search(member)
        .map_err(|_| Denial::IncompleteContributionClosure)?;
    u32::try_from(index).map_err(|_| Denial::InvalidContributionMemberOrdinal)
}
