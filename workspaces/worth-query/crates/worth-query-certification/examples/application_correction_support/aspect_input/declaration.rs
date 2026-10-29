//! Real public declaration closure, before installation can issue removal authority.
use super::super::schema::*;
use super::{authoring::EditInputAspect, Bias, InputFacts, Sibling};
use worth_query_host::facade::declaration::application_schema::*;
fn declaration(
    optional: bool,
    sibling: bool,
) -> Result<ApplicationSchemaDeclaration<NumericSchema>, ApplicationSchemaDeclarationDenial> {
    let schema = ApplicationSchemaDeclarationBuilder::<NumericSchema>::for_schema()
        .entity(Number::reference());
    let schema = if optional {
        schema.optional_aspect(Number::reference(), InputFacts::reference())
    } else {
        schema.aspect(Number::reference(), InputFacts::reference())
    };
    let schema = schema
        .field(Number::reference(), Bias::reference())
        .field(Number::reference(), Sibling::reference())
        .operation(
            EditInputAspect::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(EditInputAspect::reference(), Bias::reference());
    let schema = if sibling {
        schema.operation_read_field(EditInputAspect::reference(), Sibling::reference())
    } else {
        schema
    };
    schema
        .operation_remove_aspect(EditInputAspect::reference(), InputFacts::reference())
        .build()
}
pub fn only_complete_optional_parent_declarations_admit_removal() {
    declaration(true, true).unwrap();
    for (optional, sibling) in [(true, false), (false, true)] {
        assert!(matches!(
            declaration(optional, sibling),
            Err(
                ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency {
                    dependency: ApplicationOperationDependencyLocus::ProgramTarget(
                        ApplicationOperationProgramTarget::RemoveAspect { .. }
                    ),
                    ..
                }
            )
        ));
    }
}
