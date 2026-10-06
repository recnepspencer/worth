//! Resolution of authored type syntax against a closed schema.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResult,
};
use crate::expressions::syntax::ast::{QualifiedName, TypeArgument, TypeSyntax};

use super::dimension::{BaseDimension, ExpressionDimension};
use super::schema::ExpressionSchema;
use super::{ExpressionType, IntegerType};

const TYPE_DEPTH: u32 = 64;

/// Resolves `syntax`, reporting denials at syntax node `node`.
pub(crate) fn resolve_type(
    schema: &ExpressionSchema,
    syntax: &TypeSyntax,
    node: u32,
) -> ExpressionResult<ExpressionType> {
    let ty = resolve_at(schema, syntax, 0)
        .and_then(|ty| schema.check_type(&ty).map(|()| ty))
        .map_err(|denial| denial.with_occurrence(ExpressionOccurrence::Node(node)))?;
    Ok(ty)
}

fn resolve_at(
    schema: &ExpressionSchema,
    syntax: &TypeSyntax,
    depth: u32,
) -> ExpressionResult<ExpressionType> {
    if depth >= TYPE_DEPTH {
        return Err(invalid("type nesting exceeds the depth ceiling"));
    }
    let name = syntax.name.text();
    let arguments = syntax.arguments.as_slice();
    let scalar = match name.as_str() {
        "Bool" => Some(ExpressionType::Bool),
        "Float32" => Some(ExpressionType::Float32),
        "Float64" => Some(ExpressionType::Float64),
        "Decimal" => Some(ExpressionType::Decimal),
        "String" => Some(ExpressionType::String),
        "Bytes" => Some(ExpressionType::Bytes),
        other => IntegerType::ALL
            .into_iter()
            .find(|integer| integer.name() == other)
            .map(ExpressionType::Integer),
    };
    let structural = matches!(
        name.as_str(),
        "Option" | "List" | "Map" | "Quantity" | "Bits" | "Logic4"
    );
    if syntax.version.is_some() && (scalar.is_some() || structural) {
        return Err(invalid("only nominal types carry a version"));
    }
    if let Some(scalar) = scalar {
        return no_arguments(arguments, scalar);
    }
    let nested = |index: usize| match arguments.get(index) {
        Some(TypeArgument::Type(inner)) => resolve_at(schema, inner, depth + 1),
        _ => Err(invalid("expected a type argument")),
    };
    match (name.as_str(), arguments.len()) {
        ("Option", 1) => Ok(ExpressionType::option(nested(0)?)),
        ("List", 1) => Ok(ExpressionType::list(nested(0)?)),
        ("Map", 2) => Ok(ExpressionType::map(nested(0)?, nested(1)?)),
        ("Quantity", 1) => Ok(ExpressionType::Quantity(dimension(&arguments[0])?)),
        ("Bits", 1) => Ok(ExpressionType::Bits(width(&arguments[0])?)),
        ("Logic4", 1) => Ok(ExpressionType::Logic4(width(&arguments[0])?)),
        ("Option" | "List" | "Map" | "Quantity" | "Bits" | "Logic4", _) => {
            Err(invalid("wrong number of type arguments"))
        }
        _ => {
            let nominal = schema.nominal(&name).ok_or_else(|| {
                ExpressionDenial::new(ExpressionDenialDetail::UnknownBinding(name.clone()))
            })?;
            let declared = match &nominal {
                ExpressionType::Id(declared)
                | ExpressionType::Enum(declared)
                | ExpressionType::Record(declared) => declared.version(),
                _ => unreachable!("schema nominals are nominal types"),
            };
            if syntax.version.is_some_and(|version| version != declared) {
                return Err(ExpressionDenial::new(
                    ExpressionDenialDetail::TypeMismatch {
                        expected: nominal.to_string(),
                        found: format!("{name}@{}", syntax.version.unwrap_or_default()),
                    },
                ));
            }
            no_arguments(arguments, nominal)
        }
    }
}

fn no_arguments(
    arguments: &[TypeArgument],
    ty: ExpressionType,
) -> ExpressionResult<ExpressionType> {
    if arguments.is_empty() {
        Ok(ty)
    } else {
        Err(invalid("this type takes no type arguments"))
    }
}

fn width(argument: &TypeArgument) -> ExpressionResult<u32> {
    match argument {
        TypeArgument::Width(width) => Ok(*width),
        _ => Err(invalid("expected a bit width")),
    }
}

fn dimension(argument: &TypeArgument) -> ExpressionResult<ExpressionDimension> {
    match argument {
        TypeArgument::Type(TypeSyntax {
            name,
            arguments,
            version: None,
        }) if arguments.is_empty() => {
            if name.single() == Some("dimensionless") {
                Ok(ExpressionDimension::DIMENSIONLESS)
            } else {
                base_dimension(name).map(ExpressionDimension::base)
            }
        }
        TypeArgument::Product(factors) => {
            let mut dimension = ExpressionDimension::DIMENSIONLESS;
            for (divide, name) in factors {
                let factor = ExpressionDimension::base(base_dimension(name)?);
                let next = if *divide {
                    dimension.divide(factor)
                } else {
                    dimension.multiply(factor)
                };
                dimension = next.ok_or_else(|| {
                    ExpressionDenial::new(ExpressionDenialDetail::Bounds(
                        "dimension exponent out of range",
                    ))
                })?;
            }
            Ok(dimension)
        }
        _ => Err(invalid("expected a dimension")),
    }
}

fn base_dimension(name: &QualifiedName) -> ExpressionResult<BaseDimension> {
    name.single()
        .and_then(BaseDimension::from_name)
        .ok_or_else(|| ExpressionDenial::new(ExpressionDenialDetail::UnknownBinding(name.text())))
}

fn invalid(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(reason))
}
