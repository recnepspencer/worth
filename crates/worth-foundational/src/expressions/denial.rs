//! Closed language denial vocabulary shared by every expression phase.

use super::syntax::SourceSpan;

/// The closed family of a language denial.
///
/// Families are the machine-stable classification. The typed
/// [`ExpressionDenialDetail`] carries the specific cause.
///
/// Widths show how three families divide one concern: a width no type can
/// have, such as `Bits<0>` or a literal of the wrong length, is
/// `InvalidValue`; a valid width above the profile ceiling is
/// `ResourceExceeded`; an operation's bounds on valid widths, such as a
/// slice past the bus or a non-narrowing `truncate`, are `Bounds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionDenialFamily {
    Syntax,
    UnsupportedVersion,
    UnsupportedFeature,
    UnknownBinding,
    AmbiguousBinding,
    TypeMismatch,
    InvalidValue,
    MissingOperand,
    AbsentValue,
    Bounds,
    ArithmeticOverflow,
    ArithmeticDomain,
    DivisionByZero,
    FunctionContractMismatch,
    ResourceExceeded,
}

/// A finite resource an expression phase meters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionResource {
    SourceBytes,
    SyntaxNodes,
    SyntaxDepth,
    CallDepth,
    AdmissionWork,
    ExpandedInstructions,
    DecodedBytes,
    CanonicalBytes,
    SemanticWork,
    VisitedElements,
    InputBytes,
    OutputBytes,
    ScratchBytes,
    RetainedConsumptionBytes,
    BitWidth,
    CatalogEntries,
}

/// What the syntax layer rejected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyntaxDenial {
    UnexpectedCharacter(char),
    UnexpectedToken { expected: &'static str },
    UnexpectedEnd { expected: &'static str },
    UnterminatedString,
    InvalidEscape,
    InvalidUnicodeScalar(u32),
    MalformedNumber,
    ChainedComparison,
    ChainedEquality,
    TrailingInput,
    InvalidIdentifier,
}

/// The typed cause of a denial. Each variant belongs to exactly one family.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionDenialDetail {
    Syntax(SyntaxDenial),
    UnsupportedVersion {
        artifact: &'static str,
        found: u32,
    },
    UnsupportedFeature(&'static str),
    UnknownBinding(String),
    AmbiguousBinding(String),
    TypeMismatch {
        expected: String,
        found: String,
    },
    TypeRequired(&'static str),
    InvalidValue(&'static str),
    MissingOperand(String),
    AbsentValue,
    Bounds(&'static str),
    ArithmeticOverflow(&'static str),
    ArithmeticDomain(&'static str),
    DivisionByZero,
    FunctionContractMismatch {
        function: String,
        reason: &'static str,
    },
    ResourceExceeded {
        resource: ExpressionResource,
        limit: u64,
    },
}

impl ExpressionDenialDetail {
    pub fn family(&self) -> ExpressionDenialFamily {
        use ExpressionDenialFamily as Family;
        match self {
            Self::Syntax(_) => Family::Syntax,
            Self::UnsupportedVersion { .. } => Family::UnsupportedVersion,
            Self::UnsupportedFeature(_) => Family::UnsupportedFeature,
            Self::UnknownBinding(_) => Family::UnknownBinding,
            Self::AmbiguousBinding(_) => Family::AmbiguousBinding,
            Self::TypeMismatch { .. } | Self::TypeRequired(_) => Family::TypeMismatch,
            Self::InvalidValue(_) => Family::InvalidValue,
            Self::MissingOperand(_) => Family::MissingOperand,
            Self::AbsentValue => Family::AbsentValue,
            Self::Bounds(_) => Family::Bounds,
            Self::ArithmeticOverflow(_) => Family::ArithmeticOverflow,
            Self::ArithmeticDomain(_) => Family::ArithmeticDomain,
            Self::DivisionByZero => Family::DivisionByZero,
            Self::FunctionContractMismatch { .. } => Family::FunctionContractMismatch,
            Self::ResourceExceeded { .. } => Family::ResourceExceeded,
        }
    }
}

/// Where a denial occurred.
///
/// Source spans are provenance, never mathematical meaning. A draft built
/// without source text reports the syntax node ordinal instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionOccurrence {
    Source(SourceSpan),
    Node(u32),
}

/// A typed language denial with the occurrence that caused it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionDenial {
    detail: ExpressionDenialDetail,
    occurrence: Option<ExpressionOccurrence>,
}

impl ExpressionDenial {
    pub(crate) fn new(detail: ExpressionDenialDetail) -> Self {
        Self {
            detail,
            occurrence: None,
        }
    }

    pub(crate) fn at(detail: ExpressionDenialDetail, occurrence: ExpressionOccurrence) -> Self {
        Self {
            detail,
            occurrence: Some(occurrence),
        }
    }

    pub(crate) fn resource(resource: ExpressionResource, limit: u64) -> Self {
        Self::new(ExpressionDenialDetail::ResourceExceeded { resource, limit })
    }

    pub(crate) fn with_occurrence(mut self, occurrence: ExpressionOccurrence) -> Self {
        if self.occurrence.is_none() {
            self.occurrence = Some(occurrence);
        }
        self
    }

    pub fn family(&self) -> ExpressionDenialFamily {
        self.detail.family()
    }

    pub fn detail(&self) -> &ExpressionDenialDetail {
        &self.detail
    }

    pub fn occurrence(&self) -> Option<ExpressionOccurrence> {
        self.occurrence
    }
}

pub(crate) type ExpressionResult<T> = Result<T, ExpressionDenial>;
