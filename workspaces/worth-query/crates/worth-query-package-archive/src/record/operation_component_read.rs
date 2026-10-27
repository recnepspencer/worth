//! The same descriptive scope codec for schema declarations and installed contracts.
use super::decode_budget::RecordDecodeAttempt;
use super::sequence::{decode_sequence, write_sequence};
use crate::{
    binary_encoding::BinaryEncodingSink, binary_input::BinaryInput,
    denial::WorthQueryPackageArchiveDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationComponentReadMeaning, WorthQueryPortableOperationComponentReadParts,
};

const NESTING_DEPTH: u32 = 2;

pub(super) fn require_nesting_depth(
    maximum_depth: u32,
) -> Result<(), WorthQueryPackageArchiveDenial> {
    if maximum_depth < NESTING_DEPTH {
        return Err(WorthQueryPackageArchiveDenial::new(
            crate::denial::WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded,
        ));
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    scope: &ApplicationOperationComponentReadMeaning,
) -> Result<(), WorthQueryPackageArchiveDenial> {
    output.text(scope.operation())?;
    output.text(scope.set())?;
    output.text(scope.root())?;
    output.text(scope.member())?;
    write_sequence(
        output,
        scope.paths(),
        super::application_query::write_root_path,
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationComponentReadMeaning, WorthQueryPackageArchiveDenial> {
    budget.require_nesting_depth(NESTING_DEPTH)?;
    let operation = input.text()?.to_owned();
    let set = input.text()?.to_owned();
    let root = input.text()?.to_owned();
    let member = input.text()?.to_owned();
    let paths = decode_sequence(
        input,
        budget,
        12,
        super::application_query::decode_root_path,
    )?;
    Ok(
        ApplicationOperationComponentReadMeaning::from_untrusted_parts(
            WorthQueryPortableOperationComponentReadParts {
                operation,
                set,
                root,
                member,
                paths,
            },
        ),
    )
}
