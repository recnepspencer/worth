







#[allow(unused_imports)]
pub(crate) use {crate::expressions::reference_context::generated};









#[allow(unused_imports)]
pub(crate) use {crate::expressions::reference_context::digital};



#[allow(unused_imports)]
pub(crate) use {crate::expressions::reference_context::cel};



pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::AdmittedExpression, worth_foundational::expression_api::BaseDimension, worth_foundational::expression_api::ExpressionDenial, worth_foundational::expression_api::ExpressionDenialFamily, worth_foundational::expression_api::ExpressionDimension, worth_foundational::expression_api::ExpressionDraft, worth_foundational::expression_api::ExpressionEvaluation, worth_foundational::expression_api::ExpressionFunctionCatalog, worth_foundational::expression_api::ExpressionFunctionDeclaration, worth_foundational::expression_api::ExpressionInputs, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionSchema, worth_foundational::expression_api::ExpressionType, worth_foundational::expression_api::ExpressionTypeName, worth_foundational::expression_api::ExpressionValue, worth_foundational::expression_api::IntegerType};
pub(crate) use {worth_foundational::CanonicalF64};

pub(crate) fn length() -> ExpressionType {
    worth_foundational::expression_api::ExpressionType::Quantity(worth_foundational::expression_api::ExpressionDimension::base(worth_foundational::expression_api::BaseDimension::Length))
}

pub(crate) fn nominal(name: &str) -> ExpressionTypeName {
    worth_foundational::expression_api::ExpressionTypeName::new(name, 1).expect("fixture names are valid")
}

/// Operands shared by the suites: engineering quantities, a nominal record
/// and enum, options, collections, and digital buses.
pub(crate) fn schema() -> ExpressionSchema {
    let build = || -> Result<ExpressionSchema, ExpressionDenial> {
        Ok(worth_foundational::expression_api::ExpressionSchema::builder()
            .enumeration("Material", 1, ["Steel", "Timber"])?
            .record(
                "Frame",
                1,
                [
                    ("thickness", length()),
                    ("material", worth_foundational::expression_api::ExpressionType::Enum(nominal("Material"))),
                ],
            )?
            .operand("width", worth_foundational::expression_api::ExpressionType::Float64)?
            .operand("depth", worth_foundational::expression_api::ExpressionType::Float64)?
            .operand("clear_width", length())?
            .operand("frame", worth_foundational::expression_api::ExpressionType::Record(nominal("Frame")))?
            .operand(
                "members",
                worth_foundational::expression_api::ExpressionType::list(worth_foundational::expression_api::ExpressionType::Record(nominal("Frame"))),
            )?
            .operand("label", worth_foundational::expression_api::ExpressionType::option(worth_foundational::expression_api::ExpressionType::String))?
            .operand("count", worth_foundational::expression_api::ExpressionType::INT64)?
            .operand("small", worth_foundational::expression_api::ExpressionType::Integer(worth_foundational::expression_api::IntegerType::Int32))?
            .operand("ready", worth_foundational::expression_api::ExpressionType::Bool)?
            .operand("bus", worth_foundational::expression_api::ExpressionType::Bits(8))?
            .operand("enable", worth_foundational::expression_api::ExpressionType::Logic4(1))?
            .build())
    };
    build().expect("fixture schema is valid")
}

pub(crate) fn empty_catalog(schema: &ExpressionSchema) -> ExpressionFunctionCatalog {
    worth_foundational::expression_api::ExpressionFunctionCatalog::builder(schema, worth_foundational::expression_api::ExpressionProfile::interactive()).build()
}

pub(crate) fn admit(source: &str) -> Result<AdmittedExpression, ExpressionDenial> {
    let schema = schema();
    expressions().parse(source)?.admit(
        &schema,
        &empty_catalog(&schema),
        worth_foundational::expression_api::ExpressionProfile::interactive(),
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
    worth_foundational::expression_api::ExpressionValue::float64(worth_foundational::CanonicalF64::from_f64(value)).expect("fixture floats are finite")
}

/// A length in meters.
pub(crate) fn meters(value: f64) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::quantity(worth_foundational::CanonicalF64::from_f64(value)).expect("fixture lengths are finite")
}

/// A `Frame` record: thickness in meters and a `Material` variant.
pub(crate) fn frame(thickness: f64, material: u32) -> ExpressionValue {
    worth_foundational::expression_api::ExpressionValue::record(vec![meters(thickness), worth_foundational::expression_api::ExpressionValue::variant(material)])
}

/// Values for every fixture operand except `label`, which stays absent.
pub(crate) fn inputs() -> ExpressionInputs {
    let bind = || -> Result<ExpressionInputs, ExpressionDenial> {
        Ok(worth_foundational::expression_api::ExpressionInputs::builder(&schema())
            .bind("width", float(2.5))?
            .bind("depth", float(0.5))?
            .bind("clear_width", meters(0.9))?
            .bind("frame", frame(0.01, 0))?
            .bind(
                "members",
                worth_foundational::expression_api::ExpressionValue::list(vec![frame(0.01, 0), frame(0.02, 1), frame(0.03, 0)]),
            )?
            .bind("label", worth_foundational::expression_api::ExpressionValue::none())?
            .bind("count", worth_foundational::expression_api::ExpressionValue::integer(7))?
            .bind("small", worth_foundational::expression_api::ExpressionValue::integer(-3))?
            .bind("ready", worth_foundational::expression_api::ExpressionValue::bool(true))?
            .bind("bus", worth_foundational::expression_api::ExpressionValue::bits(8, vec![0b1010_0101])?)?
            .bind("enable", worth_foundational::expression_api::ExpressionValue::logic4(1, vec![1], vec![0])?)?
            .build())
    };
    bind().expect("fixture inputs conform")
}

pub(crate) fn evaluate(source: &str) -> ExpressionEvaluation {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source} was denied: {denial:?}"));
    admitted
        .compile()
        .evaluate(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive())
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