//! The shared typed expression language: admission semantics, canonical
//! identity, resource ceilings, grammar conformance, and evaluation.

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

#[path = "expressions/evaluation.rs"]
mod evaluation;

#[path = "expressions/stepping.rs"]
mod stepping;

#[path = "expressions/consumption.rs"]
mod consumption;

#[path = "expressions/digital.rs"]
mod digital;

#[path = "expressions/reference.rs"]
mod reference;

#[path = "expressions/cel.rs"]
mod cel;

#[path = "expressions/vectors.rs"]
mod vectors;

use worth_foundational::expression_api::{
    expressions, AdmittedExpression, BaseDimension, ExpressionDenial, ExpressionDenialFamily,
    ExpressionDimension, ExpressionDraft, ExpressionEvaluation, ExpressionFunctionCatalog,
    ExpressionFunctionDeclaration, ExpressionInputs, ExpressionProfile, ExpressionSchema,
    ExpressionType, ExpressionTypeName, ExpressionValue, IntegerType,
};
use worth_foundational::CanonicalF64;

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

pub(crate) fn float(value: f64) -> ExpressionValue {
    ExpressionValue::float64(CanonicalF64::from_f64(value)).expect("fixture floats are finite")
}

/// A length in metres.
pub(crate) fn metres(value: f64) -> ExpressionValue {
    ExpressionValue::quantity(CanonicalF64::from_f64(value)).expect("fixture lengths are finite")
}

/// A `Frame` record: thickness in metres and a `Material` variant.
pub(crate) fn frame(thickness: f64, material: u32) -> ExpressionValue {
    ExpressionValue::record(vec![metres(thickness), ExpressionValue::variant(material)])
}

/// Values for every fixture operand except `label`, which stays absent.
pub(crate) fn inputs() -> ExpressionInputs {
    let bind = || -> Result<ExpressionInputs, ExpressionDenial> {
        Ok(ExpressionInputs::builder(&schema())
            .bind("width", float(2.5))?
            .bind("depth", float(0.5))?
            .bind("clear_width", metres(0.9))?
            .bind("frame", frame(0.01, 0))?
            .bind(
                "members",
                ExpressionValue::list(vec![frame(0.01, 0), frame(0.02, 1), frame(0.03, 0)]),
            )?
            .bind("label", ExpressionValue::none())?
            .bind("count", ExpressionValue::integer(7))?
            .bind("small", ExpressionValue::integer(-3))?
            .bind("ready", ExpressionValue::bool(true))?
            .bind("bus", ExpressionValue::bits(8, vec![0b1010_0101])?)?
            .bind("enable", ExpressionValue::logic4(1, vec![1], vec![0])?)?
            .build())
    };
    bind().expect("fixture inputs conform")
}

pub(crate) fn evaluate(source: &str) -> ExpressionEvaluation {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source} was denied: {denial:?}"));
    admitted
        .compile()
        .evaluate(&inputs(), &ExpressionProfile::interactive())
}

pub(crate) fn value(source: &str) -> ExpressionValue {
    match evaluate(source).into_result() {
        Ok(value) => value,
        Err(denial) => panic!("{source} denied at evaluation: {denial:?}"),
    }
}

pub(crate) fn evaluation_denial(source: &str) -> ExpressionDenial {
    match evaluate(source).into_result() {
        Ok(value) => panic!("{source} evaluated to {value:?}"),
        Err(denial) => denial,
    }
}
