use super::ApplicationOperationComponentReadMeaning;
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};

pub(in crate::application_schema) fn validate_identifiers(
    meaning: &ApplicationOperationComponentReadMeaning,
) -> Result<(), Denial> {
    for name in [
        meaning.operation(),
        meaning.set(),
        meaning.root(),
        meaning.member(),
    ] {
        super::super::identifier_validation::validate_simple_identifier(name)?;
    }
    Ok(())
}

pub(in crate::application_schema) fn validate_closure(
    meaning: &ApplicationOperationComponentReadMeaning,
    members: &[Member],
) -> Result<(), Denial> {
    let set_matches = members.iter().any(|member| {
        matches!(member,
        Member::ComponentSet { definition }
            if definition.name() == meaning.set() && definition.member() == meaning.member())
    });
    let operation_matches = members.iter().any(|member| {
        matches!(member,
        Member::ApplicationMutation { description }
            if description.operation() == meaning.operation()
                && description.scope().entity == meaning.root())
    });
    let paths = meaning.paths();
    if !set_matches
        || !operation_matches
        || paths.is_empty()
        || !paths.windows(2).all(|pair| pair[0] < pair[1])
        || paths.iter().any(|path| {
            path.start_entity() != meaning.root()
                || path.terminal_entity() != meaning.member()
                || !super::super::root_path_closure::is_closed(path, members)
        })
    {
        return Err(Denial::InvalidOperationComponentRead);
    }
    Ok(())
}
