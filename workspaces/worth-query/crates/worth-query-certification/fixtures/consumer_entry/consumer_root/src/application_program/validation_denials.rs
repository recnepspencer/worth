use worth_query_decl::facade::application_program::{
    ApplicationProgramAuthoring, ApplicationProgramValidationDenialKind,
};

use super::{
    ConsumerSchema, DuplicateFeatureProgram, MissingRequiredInputProgram, UndeclaredInputProgram,
    UnexportedCrossInstanceProgram,
};

pub fn missing_required_input_is_denied() -> ApplicationProgramValidationDenialKind {
    match ApplicationProgramAuthoring::<ConsumerSchema, MissingRequiredInputProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("an unconnected required input was accepted"),
        Err(denial) => denial.kind(),
    }
}

pub fn duplicate_feature_is_denied() -> ApplicationProgramValidationDenialKind {
    match ApplicationProgramAuthoring::<ConsumerSchema, DuplicateFeatureProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("a duplicate feature declaration was accepted"),
        Err(denial) => denial.kind(),
    }
}

pub fn undeclared_input_is_denied() -> ApplicationProgramValidationDenialKind {
    match ApplicationProgramAuthoring::<ConsumerSchema, UndeclaredInputProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("a connection to an undeclared feature input was accepted"),
        Err(denial) => denial.kind(),
    }
}

pub fn unexported_cross_instance_is_denied() -> ApplicationProgramValidationDenialKind {
    match ApplicationProgramAuthoring::<ConsumerSchema, UnexportedCrossInstanceProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("an unexported cross-instance binding was accepted"),
        Err(denial) => denial.kind(),
    }
}
