//! The versioned V1 unit catalog.
//!
//! A unit has a dimension and an exact scale to canonical SI. Scales are exact
//! rationals, optionally times an integer power of the degree constant (the
//! correctly rounded binary64 value of pi/180), so every conversion rounds once.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResult,
};
use crate::expressions::syntax::ast::{BinaryOp, NodeId, SyntaxNode, SyntaxTree};

use super::dimension::{BaseDimension, ExpressionDimension};

/// The unit catalog meaning version bound into program identity.
pub(crate) const UNIT_CATALOG_VERSION: &str = "worth-units-v1";

/// The correctly rounded binary64 value of pi/180.
pub(crate) const DEGREE_IN_RADIANS: f64 = std::f64::consts::PI / 180.0;

/// `numerator / denominator * DEGREE_IN_RADIANS^degree_power`, in lowest terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct UnitScale {
    pub(crate) numerator: u128,
    pub(crate) denominator: u128,
    pub(crate) degree_power: i8,
}

impl UnitScale {
    const ONE: Self = Self::ratio(1, 1);

    const fn ratio(numerator: u128, denominator: u128) -> Self {
        Self {
            numerator,
            denominator,
            degree_power: 0,
        }
    }

    fn reduced(numerator: u128, denominator: u128, degree_power: i8) -> Self {
        let divisor = gcd(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
            degree_power,
        }
    }

    fn multiply(self, other: Self) -> Option<Self> {
        let left = gcd(self.numerator, other.denominator);
        let right = gcd(other.numerator, self.denominator);
        let numerator = (self.numerator / left).checked_mul(other.numerator / right)?;
        let denominator = (self.denominator / right).checked_mul(other.denominator / left)?;
        let degree_power = self.degree_power.checked_add(other.degree_power)?;
        Some(Self::reduced(numerator, denominator, degree_power))
    }

    fn reciprocal(self) -> Option<Self> {
        Some(Self {
            numerator: self.denominator,
            denominator: self.numerator,
            degree_power: self.degree_power.checked_neg()?,
        })
    }
}

const fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

/// A resolved unit: its dimension and exact scale to canonical SI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Unit {
    pub(crate) dimension: ExpressionDimension,
    pub(crate) scale: UnitScale,
}

//                        L  M  T  I  Θ  N  J  A
const LENGTH: [i8; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
const MASS: [i8; 8] = [0, 1, 0, 0, 0, 0, 0, 0];
const TIME: [i8; 8] = [0, 0, 1, 0, 0, 0, 0, 0];
const CURRENT: [i8; 8] = [0, 0, 0, 1, 0, 0, 0, 0];
const FORCE: [i8; 8] = [1, 1, -2, 0, 0, 0, 0, 0];
const PRESSURE: [i8; 8] = [-1, 1, -2, 0, 0, 0, 0, 0];
const ENERGY: [i8; 8] = [2, 1, -2, 0, 0, 0, 0, 0];
const POWER: [i8; 8] = [2, 1, -3, 0, 0, 0, 0, 0];
const FREQUENCY: [i8; 8] = [0, 0, -1, 0, 0, 0, 0, 0];
const CHARGE: [i8; 8] = [0, 0, 1, 1, 0, 0, 0, 0];
const VOLTAGE: [i8; 8] = [2, 1, -3, -1, 0, 0, 0, 0];

const fn unit(name: &'static str, exponents: [i8; 8], scale: UnitScale) -> (&'static str, Unit) {
    (
        name,
        Unit {
            dimension: ExpressionDimension::from_catalog(exponents),
            scale,
        },
    )
}

const fn base(name: &'static str, base: BaseDimension) -> (&'static str, Unit) {
    (
        name,
        Unit {
            dimension: ExpressionDimension::base(base),
            scale: UnitScale::ONE,
        },
    )
}

const THOUSANDTH: UnitScale = UnitScale::ratio(1, 1_000);
const MILLIONTH: UnitScale = UnitScale::ratio(1, 1_000_000);
const BILLIONTH: UnitScale = UnitScale::ratio(1, 1_000_000_000);

/// The closed V1 catalog. Changing any row is a new catalog version.
const CATALOG: [(&str, Unit); 38] = [
    base("m", BaseDimension::Length),
    unit("km", LENGTH, UnitScale::ratio(1_000, 1)),
    unit("cm", LENGTH, UnitScale::ratio(1, 100)),
    unit("mm", LENGTH, THOUSANDTH),
    unit("um", LENGTH, MILLIONTH),
    unit("nm", LENGTH, BILLIONTH),
    unit("in", LENGTH, UnitScale::ratio(127, 5_000)),
    unit("ft", LENGTH, UnitScale::ratio(381, 1_250)),
    base("kg", BaseDimension::Mass),
    unit("g", MASS, THOUSANDTH),
    unit("mg", MASS, MILLIONTH),
    base("s", BaseDimension::Time),
    unit("ms", TIME, THOUSANDTH),
    unit("us", TIME, MILLIONTH),
    unit("ns", TIME, BILLIONTH),
    unit("min", TIME, UnitScale::ratio(60, 1)),
    unit("h", TIME, UnitScale::ratio(3_600, 1)),
    base("A", BaseDimension::Current),
    unit("mA", CURRENT, THOUSANDTH),
    base("K", BaseDimension::Temperature),
    base("mol", BaseDimension::Amount),
    base("cd", BaseDimension::LuminousIntensity),
    base("rad", BaseDimension::Angle),
    (
        "deg",
        Unit {
            dimension: ExpressionDimension::base(BaseDimension::Angle),
            scale: UnitScale {
                numerator: 1,
                denominator: 1,
                degree_power: 1,
            },
        },
    ),
    unit("N", FORCE, UnitScale::ONE),
    unit("kN", FORCE, UnitScale::ratio(1_000, 1)),
    unit("Pa", PRESSURE, UnitScale::ONE),
    unit("kPa", PRESSURE, UnitScale::ratio(1_000, 1)),
    unit("MPa", PRESSURE, UnitScale::ratio(1_000_000, 1)),
    unit("GPa", PRESSURE, UnitScale::ratio(1_000_000_000, 1)),
    unit("J", ENERGY, UnitScale::ONE),
    unit("kJ", ENERGY, UnitScale::ratio(1_000, 1)),
    unit("W", POWER, UnitScale::ONE),
    unit("kW", POWER, UnitScale::ratio(1_000, 1)),
    unit("Hz", FREQUENCY, UnitScale::ONE),
    unit("kHz", FREQUENCY, UnitScale::ratio(1_000, 1)),
    unit("C", CHARGE, UnitScale::ONE),
    unit("V", VOLTAGE, UnitScale::ONE),
];

pub(crate) fn catalog_unit(name: &str) -> Option<Unit> {
    CATALOG
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, unit)| *unit)
}

/// Interprets a unit expression: catalog names, `1`, `*`, `/`, and
/// parentheses. Recursion is bounded by the admitted syntax depth.
pub(crate) fn resolve_unit(tree: &SyntaxTree, node: NodeId) -> ExpressionResult<Unit> {
    let deny = |detail| ExpressionDenial::at(detail, ExpressionOccurrence::Node(node.0));
    match tree.node(node) {
        SyntaxNode::Integer(1) => Ok(Unit {
            dimension: ExpressionDimension::DIMENSIONLESS,
            scale: UnitScale::ONE,
        }),
        SyntaxNode::Name(name) => name
            .single()
            .and_then(catalog_unit)
            .ok_or_else(|| deny(ExpressionDenialDetail::UnknownBinding(name.text()))),
        SyntaxNode::Binary { op, left, right }
            if matches!(op, BinaryOp::Multiply | BinaryOp::Divide) =>
        {
            let left = resolve_unit(tree, *left)?;
            let mut right = resolve_unit(tree, *right)?;
            if *op == BinaryOp::Divide {
                right.scale = right.scale.reciprocal().ok_or_else(|| {
                    deny(ExpressionDenialDetail::Bounds("unit scale out of range"))
                })?;
                right.dimension = ExpressionDimension::DIMENSIONLESS
                    .divide(right.dimension)
                    .ok_or_else(|| {
                        deny(ExpressionDenialDetail::Bounds(
                            "dimension exponent out of range",
                        ))
                    })?;
            }
            let dimension = left.dimension.multiply(right.dimension).ok_or_else(|| {
                deny(ExpressionDenialDetail::Bounds(
                    "dimension exponent out of range",
                ))
            })?;
            let scale = left
                .scale
                .multiply(right.scale)
                .ok_or_else(|| deny(ExpressionDenialDetail::Bounds("unit scale out of range")))?;
            Ok(Unit { dimension, scale })
        }
        _ => Err(deny(ExpressionDenialDetail::UnsupportedFeature(
            "unit expressions use catalog names, `1`, `*`, `/`, and parentheses",
        ))),
    }
}
