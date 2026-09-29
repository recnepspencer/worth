use std::sync::Arc;

use worth_foundational::expression_api::{
    ExpressionConsumption, ExpressionCost, ExpressionProgramIdentity,
};
use worth_ui_dsl::WorthUiDslSourceSpan;
use worth_ui_query_binding::UiProjectionInputFactReference;

use super::UiExpressionOutcome;
use crate::runtime::expression::UiExpressionSlot;
use crate::runtime::intent::UiIntentApplicationInputReference;
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// The exact fact one operand was read from. Query and application facts keep
/// the whole reference the owner issued, so currentness compares everything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionOperandFact {
    Query(UiProjectionInputFactReference),
    Application(UiIntentApplicationInputReference),
    Expression {
        slot: UiExpressionSlot,
        outcome_revision: u64,
    },
    /// The owner had no fact for this operand.
    Absent {
        operand: Box<str>,
    },
}

/// One retained evaluation of one expression.
#[derive(Clone, Debug)]
pub struct UiExpressionEvaluationRecord {
    pub(super) slot: UiExpressionSlot,
    pub(super) identity: Box<str>,
    pub(super) outcome: UiExpressionOutcome,
    pub(super) outcome_revision: u64,
    pub(super) operands: Box<[UiExpressionOperandFact]>,
    pub(super) consumed: Option<ExpressionConsumption>,
    pub(super) program_identity: ExpressionProgramIdentity,
    pub(super) generation: WorthUiActiveApplicationGenerationIdentity,
    pub(super) cost: Option<ExpressionCost>,
    pub(super) span: Option<Arc<WorthUiDslSourceSpan>>,
}

/// The identity of one published expression result. It stays current only
/// while its generation and outcome revision are the retained ones.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiExpressionResultReference {
    pub(super) generation: WorthUiActiveApplicationGenerationIdentity,
    pub(super) slot: UiExpressionSlot,
    pub(super) identity: Box<str>,
    pub(super) outcome_revision: u64,
    pub(super) outcome: UiExpressionOutcome,
}

impl UiExpressionEvaluationRecord {
    pub const fn slot(&self) -> UiExpressionSlot {
        self.slot
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn outcome(&self) -> &UiExpressionOutcome {
        &self.outcome
    }

    /// Bumped whenever the outcome, value or posture, changes.
    pub const fn outcome_revision(&self) -> u64 {
        self.outcome_revision
    }

    /// The facts read, one per operand in name order.
    pub fn operands(&self) -> &[UiExpressionOperandFact] {
        &self.operands
    }

    /// The reads the kernel reports the outcome depended on. `None` when a
    /// non-current operand settled the expression without evaluating it.
    pub const fn consumed(&self) -> Option<&ExpressionConsumption> {
        self.consumed.as_ref()
    }

    pub const fn program_identity(&self) -> &ExpressionProgramIdentity {
        &self.program_identity
    }

    pub const fn generation(&self) -> &WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub const fn cost(&self) -> Option<&ExpressionCost> {
        self.cost.as_ref()
    }

    /// Where the expression text sits in its host file. Evidence only: it is
    /// never part of identity or of the change test.
    pub fn span(&self) -> Option<&WorthUiDslSourceSpan> {
        self.span.as_deref()
    }

    pub(super) fn result_reference(&self) -> UiExpressionResultReference {
        UiExpressionResultReference {
            generation: self.generation.clone(),
            slot: self.slot,
            identity: self.identity.clone(),
            outcome_revision: self.outcome_revision,
            outcome: self.outcome.clone(),
        }
    }
}

impl UiExpressionResultReference {
    pub const fn generation(&self) -> &WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub const fn slot(&self) -> UiExpressionSlot {
        self.slot
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn outcome_revision(&self) -> u64 {
        self.outcome_revision
    }

    pub const fn outcome(&self) -> &UiExpressionOutcome {
        &self.outcome
    }
}
