//! The shared typed expression language: admission semantics, canonical
//! identity, resource ceilings, and grammar conformance.

#[path = "expressions/semantics.rs"]
mod semantics;

#[path = "expressions/canonical.rs"]
mod canonical;

#[path = "expressions/resources.rs"]
mod resources;

#[path = "expressions/conformance.rs"]
mod conformance;

#[path = "expressions/generated.rs"]
mod generated;

#[path = "expressions/denials.rs"]
mod denials;

use worth_foundational::expression_api::{
    expressions, AdmittedExpression, BaseDimension, ExpressionDenial, ExpressionDenialFamily,
    ExpressionDimension, ExpressionDraft, ExpressionFunctionCatalog, ExpressionFunctionDeclaration,
    ExpressionProfile, ExpressionSchema, ExpressionType, ExpressionTypeName, IntegerType,
};

pub(crate) fn length() -> ExpressionType {
    ExpressionType::Quantity(ExpressionDimension::base(BaseDimension::Length))
}

pub(crate) fn nominal(name: &str) -> ExpressionTypeName {
    ExpressionTypeName::new(name, 1).expect("fixture names are valid")
}

/// Operands shared by the suites: engineering quantities, a nominal record
/// and enum, options, collections, and digital buses.
pub(crate) fn schema() -> ExpressionSchema {
    let build = || -> Result<ExpressionSchema, ExpressionDenial> {
        Ok(ExpressionSchema::builder()
            .enumeration("Material", 1, ["Steel", "Timber"])?
            .record(
                "Frame",
                1,
                [
                    ("thickness", length()),
                    ("material", ExpressionType::Enum(nominal("Material"))),
                ],
            )?
            .operand("width", ExpressionType::Float64)?
            .operand("depth", ExpressionType::Float64)?
            .operand("clear_width", length())?
            .operand("frame", ExpressionType::Record(nominal("Frame")))?
            .operand(
                "members",
                ExpressionType::list(ExpressionType::Record(nominal("Frame"))),
            )?
            .operand("label", ExpressionType::option(ExpressionType::String))?
            .operand("count", ExpressionType::INT64)?
            .operand("small", ExpressionType::Integer(IntegerType::Int32))?
            .operand("ready", ExpressionType::Bool)?
            .operand("bus", ExpressionType::Bits(8))?
            .operand("enable", ExpressionType::Logic4(1))?
            .build())
    };
    build().expect("fixture schema is valid")
}

pub(crate) fn empty_catalog(schema: &ExpressionSchema) -> ExpressionFunctionCatalog {
    ExpressionFunctionCatalog::builder(schema, ExpressionProfile::interactive()).build()
}

pub(crate) fn admit(source: &str) -> Result<AdmittedExpression, ExpressionDenial> {
    let schema = schema();
    expressions().parse(source)?.admit(
        &schema,
        &empty_catalog(&schema),
        ExpressionProfile::interactive(),
    )
}

pub(crate) fn admitted_type(source: &str) -> ExpressionType {
    match admit(source) {
        Ok(admitted) => admitted.result_type().clone(),
        Err(denial) => panic!("{source} was denied: {denial:?}"),
    }
}

pub(crate) fn denied_family(source: &str) -> ExpressionDenialFamily {
    match admit(source) {
        Ok(admitted) => panic!("{source} was admitted as {:?}", admitted.result_type()),
        Err(denial) => denial.family(),
    }
}

pub(crate) fn draft(source: &str) -> ExpressionDraft {
    expressions().parse(source).expect("fixture source parses")
}

pub(crate) fn function(
    name: &str,
    parameters: &[(&str, ExpressionType)],
    result: ExpressionType,
    body: &str,
) -> ExpressionFunctionDeclaration {
    ExpressionFunctionDeclaration {
        name: nominal(name),
        parameters: parameters
            .iter()
            .map(|(name, ty)| ((*name).into(), ty.clone()))
            .collect(),
        result,
        body: draft(body),
    }
}
