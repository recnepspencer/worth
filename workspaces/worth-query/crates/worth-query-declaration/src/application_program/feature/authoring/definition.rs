//! Static feature membership before its canonical declaration is erased.
use crate::application_program::{ApplicationFeatureSpec, ApplicationProgramOutputRootsShape};
use crate::application_schema::{ApplicationSchema, ApplicationSchemaDeclarationBuilder};

/// A selected feature definition projects its authored members, not a runtime registry.
///
/// A tuple of 2 to 12 definitions is itself a definition: its members and
/// feature specs follow tuple order and its outputs nest as right-leaning
/// pairs, so `(A, B, C)` composes exactly as `(A, (B, C))` without the
/// caller nesting it.
pub trait ApplicationFeatureDefinition<S: ApplicationSchema> {
    type Outputs: ApplicationProgramOutputRootsShape<S>;
    fn declare_members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema.in_feature(std::any::type_name::<Self>(), Self::members)
    }

    fn members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S>;
    fn feature_specs() -> Vec<ApplicationFeatureSpec>;
}

/// The tail after a tuple's head: the member itself, or the shorter tuple.
macro_rules! feature_definition_tail {
    ($Only:ident) => { $Only };
    ($($Rest:ident),+) => { ($($Rest),+) };
}

/// One tuple arity: the head's outputs pair with the tail's.
macro_rules! feature_definition_tuple {
    ($First:ident $(, $Rest:ident)+) => {
        impl<S, $First, $($Rest),+> ApplicationFeatureDefinition<S> for ($First, $($Rest),+)
        where
            S: ApplicationSchema,
            $First: ApplicationFeatureDefinition<S>,
            $($Rest: ApplicationFeatureDefinition<S>,)+
        {
            type Outputs = (
                <$First as ApplicationFeatureDefinition<S>>::Outputs,
                <feature_definition_tail!($($Rest),+) as ApplicationFeatureDefinition<S>>::Outputs,
            );
            fn members(
                schema: ApplicationSchemaDeclarationBuilder<S>,
            ) -> ApplicationSchemaDeclarationBuilder<S> {
                let schema = $First::declare_members(schema);
                $(let schema = $Rest::declare_members(schema);)+
                schema
            }
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                let mut specs = $First::feature_specs();
                $(specs.extend($Rest::feature_specs());)+
                specs
            }
        }
    };
}

/// Every arity from the full list down to a pair.
macro_rules! feature_definition_tuples {
    ($First:ident, $Second:ident) => {
        feature_definition_tuple!($First, $Second);
    };
    ($First:ident $(, $Rest:ident)+) => {
        feature_definition_tuple!($First $(, $Rest)+);
        feature_definition_tuples!($($Rest),+);
    };
}

feature_definition_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
