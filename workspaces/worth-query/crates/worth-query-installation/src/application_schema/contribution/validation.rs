use worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryApplicationContributionCompilationDenial {
    MemberOrdinalOutOfBounds,
}

/// Confirms that declaration-owned contribution ordinals resolve against the
/// exact canonical member array retained by installation.
///
/// Declaration remains the authority for identity, ordering, overlap and
/// closure. Installation does not reconstruct or recanonicalize that meaning.
pub(crate) fn validate_installed_contribution_members(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<(), WorthQueryApplicationContributionCompilationDenial> {
    for contribution in declaration.contributions() {
        for ordinal in contribution.member_ordinals() {
            let index = usize::try_from(*ordinal).map_err(|_| {
                WorthQueryApplicationContributionCompilationDenial::MemberOrdinalOutOfBounds
            })?;
            if declaration.members().get(index).is_none() {
                return Err(
                    WorthQueryApplicationContributionCompilationDenial::MemberOrdinalOutOfBounds,
                );
            }
        }
    }
    Ok(())
}
