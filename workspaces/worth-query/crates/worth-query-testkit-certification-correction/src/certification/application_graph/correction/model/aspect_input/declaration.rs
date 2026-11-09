//! Real public declaration closure, before installation can issue removal authority.
use super::super::schema::*;
use super::{authoring::EditInputAspect, InputFacts};
use worth_query_host::facade::declaration::application_schema::fragment::{self, *};
mod model;
use model::{OptionalModel, OptionalNumber, RequiredModel, RequiredNumber};
use worth_query_host::facade::declaration::application_schema::*;
fn declaration(
    optional: bool,
    sibling: bool,
) -> Result<ApplicationSchemaDeclaration<NumericSchema>, ApplicationSchemaDeclarationDenial> {
    if optional {
        finish(
            ApplicationSchemaDeclarationBuilder::from_fragment(OptionalModel)?,
            sibling,
            FieldSelection::<OptionalModel, OptionalNumber, InputFacts, u64>::at("Bias"),
            FieldSelection::<OptionalModel, OptionalNumber, InputFacts, u64>::at("Sibling"),
            aspect(OptionalModel, OptionalNumber, InputFacts),
        )
    } else {
        finish(
            ApplicationSchemaDeclarationBuilder::from_fragment(RequiredModel)?,
            sibling,
            FieldSelection::<RequiredModel, RequiredNumber, InputFacts, u64>::at("Bias"),
            FieldSelection::<RequiredModel, RequiredNumber, InputFacts, u64>::at("Sibling"),
            aspect(RequiredModel, RequiredNumber, InputFacts),
        )
    }
}
fn finish<F: Fragment<Bindings = NumericModelBindings>, E: Entity>(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    sibling: bool,
    bias: FieldSelection<F, E, InputFacts, u64>,
    sibling_field: FieldSelection<F, E, InputFacts, u64>,
    parent: AspectSelection<F, E, InputFacts>,
) -> Result<ApplicationSchemaDeclaration<NumericSchema>, ApplicationSchemaDeclarationDenial> {
    let schema = schema
        .operation(
            EditInputAspect::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(EditInputAspect::reference(), bias);
    let schema = if sibling {
        schema.operation_read_field(EditInputAspect::reference(), sibling_field)
    } else {
        schema
    };
    schema
        .operation_remove_aspect(EditInputAspect::reference(), parent)
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
