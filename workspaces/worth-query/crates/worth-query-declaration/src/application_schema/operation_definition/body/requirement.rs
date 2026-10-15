//! Installed expression meaning and typed sources for an operation precondition.
mod authoring;
mod encoding;
mod expression;
mod operands;
mod principal;
mod validation;

pub use authoring::ApplicationOperationBodyRequirementOperands;
pub use expression::ApplicationOperationBodyRequirementExpression;
pub use operands::{
    ApplicationOperationBodyRequirementDirection, ApplicationOperationBodyRequirementOperand,
    ApplicationOperationBodyRequirementSource,
};
pub(super) use validation::validate_requirements;

use std::marker::PhantomData;
use worth_foundational::expression_api::{ExpressionDenial, ExpressionDraft};

/// A requirement is admitted as pure Boolean meaning before body attachment.
pub struct ApplicationOperationBodyRequirement<Schema, Operation> {
    pub(super) program: ApplicationOperationBodyRequirementProgram,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

/// Why a requirement's names, operands or expressions were refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationOperationBodyRequirementDenial {
    InvalidName,
    DuplicateOperand,
    DuplicateRequirement,
    MissingTarget,
    UnusedOperand,
    ForeignScope,
    UnsupportedScalar,
    Expression(ExpressionDenial),
}

impl From<ExpressionDenial> for ApplicationOperationBodyRequirementDenial {
    fn from(value: ExpressionDenial) -> Self {
        Self::Expression(value)
    }
}

/// Portable descriptive meaning. Fresh schema admission validates every source.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyRequirementProgram {
    identity: String,
    target: String,
    operands: Vec<ApplicationOperationBodyRequirementOperand>,
    applicability: Option<ApplicationOperationBodyRequirementExpression>,
    satisfaction: ApplicationOperationBodyRequirementExpression,
}

impl<Schema, Operation> ApplicationOperationBodyRequirement<Schema, Operation> {
    /// Admits Foundation drafts over typed native sources. `target` names one
    /// of those sources for guidance; it is not an independent UI path.
    pub fn admit(
        identity: &str,
        operands: ApplicationOperationBodyRequirementOperands<Schema, Operation>,
        applicability: Option<ExpressionDraft>,
        satisfaction: ExpressionDraft,
        target: &str,
    ) -> Result<Self, ApplicationOperationBodyRequirementDenial> {
        Ok(Self {
            program: ApplicationOperationBodyRequirementProgram::from_untrusted(
                identity.to_owned(),
                operands.operands,
                applicability.map(|draft| draft.encode()),
                satisfaction.encode(),
                target.to_owned(),
            )?,
            marker: PhantomData,
        })
    }
}

impl ApplicationOperationBodyRequirementProgram {
    /// Decodes pure expressions, granting no read or execution authority.
    pub fn from_untrusted(
        identity: String,
        mut operands: Vec<ApplicationOperationBodyRequirementOperand>,
        applicability: Option<Vec<u8>>,
        satisfaction: Vec<u8>,
        target: String,
    ) -> Result<Self, ApplicationOperationBodyRequirementDenial> {
        use ApplicationOperationBodyRequirementDenial as Denial;
        if identity.is_empty() || identity.trim() != identity {
            return Err(Denial::InvalidName);
        }
        operands.sort_by(|left, right| left.name().cmp(right.name()));
        if operands
            .windows(2)
            .any(|pair| pair[0].name() == pair[1].name())
        {
            return Err(Denial::DuplicateOperand);
        }
        if !operands.iter().any(|operand| operand.name() == target) {
            return Err(Denial::MissingTarget);
        }
        let applicability = applicability
            .map(|bytes| ApplicationOperationBodyRequirementExpression::decode(&bytes, &operands))
            .transpose()?;
        let satisfaction =
            ApplicationOperationBodyRequirementExpression::decode(&satisfaction, &operands)?;
        if operands.iter().any(|operand| {
            !satisfaction.reads(operand.name())
                && !applicability
                    .as_ref()
                    .is_some_and(|expression| expression.reads(operand.name()))
        }) {
            return Err(Denial::UnusedOperand);
        }
        let scope = operands
            .first()
            .map(|operand| operand.source().scope_entity());
        if operands
            .iter()
            .any(|operand| Some(operand.source().scope_entity()) != scope)
        {
            return Err(Denial::ForeignScope);
        }
        Ok(Self {
            identity,
            target,
            operands,
            applicability,
            satisfaction,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn operands(&self) -> &[ApplicationOperationBodyRequirementOperand] {
        &self.operands
    }
    pub fn applicability(&self) -> Option<&ApplicationOperationBodyRequirementExpression> {
        self.applicability.as_ref()
    }
    pub fn satisfaction(&self) -> &ApplicationOperationBodyRequirementExpression {
        &self.satisfaction
    }
    pub fn target_name(&self) -> &str {
        &self.target
    }
    pub fn target(&self) -> &ApplicationOperationBodyRequirementSource {
        self.operand(&self.target)
            .expect("target admitted with operands")
            .source()
    }
    pub fn operand(&self, name: &str) -> Option<&ApplicationOperationBodyRequirementOperand> {
        self.operands
            .binary_search_by(|operand| operand.name().cmp(name))
            .ok()
            .map(|index| &self.operands[index])
    }
}
