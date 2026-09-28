//! A workflow condition: an admitted Bool expression over named query
//! operands. True selects `ConditionSatisfied`, false `ConditionUnsatisfied`;
//! a denied evaluation selects neither.

use std::sync::Arc;

use worth_foundational::expression_api::{
    expressions, AdmittedExpression, ExpressionDenial, ExpressionDraft, ExpressionFunctionCatalog,
    ExpressionProfile, ExpressionProgramIdentity, ExpressionSchema, ExpressionType,
};

use super::operands::ApplicationWorkflowConditionOperand;

/// Why a condition could not be admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowConditionDenial {
    /// A condition reads at least one query result.
    NoOperands,
    /// The source, draft, or operand types were not admitted as a Bool
    /// expression.
    Expression(ExpressionDenial),
}

impl std::fmt::Display for ApplicationWorkflowConditionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoOperands => formatter.write_str("a workflow condition reads no query result"),
            Self::Expression(denial) => {
                write!(
                    formatter,
                    "workflow condition expression denied: {denial:?}"
                )
            }
        }
    }
}

impl std::error::Error for ApplicationWorkflowConditionDenial {}

impl From<ExpressionDenial> for ApplicationWorkflowConditionDenial {
    fn from(denial: ExpressionDenial) -> Self {
        Self::Expression(denial)
    }
}

/// An admitted condition. Equal conditions have the same encoded draft and
/// the same operands; the program identity is the canonical meaning.
#[derive(Clone, Debug)]
pub struct ApplicationWorkflowCondition {
    operands: Arc<[ApplicationWorkflowConditionOperand]>,
    draft: Arc<[u8]>,
    schema: ExpressionSchema,
    admitted: Arc<AdmittedExpression>,
}

impl PartialEq for ApplicationWorkflowCondition {
    fn eq(&self, other: &Self) -> bool {
        self.operands == other.operands
            && self.draft == other.draft
            && self.admitted.identity() == other.admitted.identity()
    }
}

impl Eq for ApplicationWorkflowCondition {}

impl ApplicationWorkflowCondition {
    /// The ceilings every condition is read, admitted, and evaluated under.
    pub const PROFILE: ExpressionProfile = ExpressionProfile::interactive();

    /// Admits `source` as a Bool expression over `operands`.
    pub fn parse(
        source: &str,
        operands: Vec<ApplicationWorkflowConditionOperand>,
    ) -> Result<Self, ApplicationWorkflowConditionDenial> {
        let draft = expressions().reading_within(Self::PROFILE).parse(source)?;
        Self::admit(&draft, operands)
    }

    /// Readmits an encoded draft over `operands`, as an untrusted artifact.
    pub fn decode(
        draft: &[u8],
        operands: Vec<ApplicationWorkflowConditionOperand>,
    ) -> Result<Self, ApplicationWorkflowConditionDenial> {
        let draft = expressions().reading_within(Self::PROFILE).decode(draft)?;
        Self::admit(&draft, operands)
    }

    fn admit(
        draft: &ExpressionDraft,
        mut operands: Vec<ApplicationWorkflowConditionOperand>,
    ) -> Result<Self, ApplicationWorkflowConditionDenial> {
        if operands.is_empty() {
            return Err(ApplicationWorkflowConditionDenial::NoOperands);
        }
        operands.sort_by(|left, right| left.name().cmp(right.name()));
        let schema = operands
            .iter()
            .try_fold(ExpressionSchema::builder(), |schema, operand| {
                schema.operand(operand.name(), operand.query().expression_type().clone())
            })?
            .build();
        let catalog = ExpressionFunctionCatalog::builder(&schema, Self::PROFILE).build();
        let admitted = draft.admit_as(&schema, &catalog, Self::PROFILE, &ExpressionType::Bool)?;
        Ok(Self {
            operands: operands.into(),
            draft: draft.encode().into(),
            schema,
            admitted: Arc::new(admitted),
        })
    }

    /// Operands in name order.
    pub fn operands(&self) -> &[ApplicationWorkflowConditionOperand] {
        &self.operands
    }

    pub fn operand(&self, name: &str) -> Option<&ApplicationWorkflowConditionOperand> {
        self.operands
            .binary_search_by(|operand| operand.name().cmp(name))
            .ok()
            .map(|index| &self.operands[index])
    }

    /// The versioned draft encoding, without source spans.
    pub fn draft(&self) -> &[u8] {
        &self.draft
    }

    /// The operand types evaluation binds values against.
    pub const fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }

    pub fn expression(&self) -> &AdmittedExpression {
        &self.admitted
    }

    pub fn identity(&self) -> &ExpressionProgramIdentity {
        self.admitted.identity()
    }
}
