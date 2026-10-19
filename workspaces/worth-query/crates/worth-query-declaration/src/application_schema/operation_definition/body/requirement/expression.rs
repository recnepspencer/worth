//! Requirements preserve an explicit Boolean result fence over the shared compiler.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial, ApplicationOperationBodyRequirementOperand,
};
use crate::application_schema::operation_definition::body::ApplicationOperationBodyExpression;
use worth_foundational::expression_api::{
    CompiledExpression, ExpressionProfile, ExpressionSchema, ExpressionType,
};
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyRequirementExpression(ApplicationOperationBodyExpression);
impl ApplicationOperationBodyRequirementExpression {
    pub const PROFILE: ExpressionProfile = ApplicationOperationBodyExpression::PROFILE;
    pub(super) fn decode(
        bytes: &[u8],
        operands: &[ApplicationOperationBodyRequirementOperand],
    ) -> Result<Self, Denial> {
        ApplicationOperationBodyExpression::decode(bytes, operands, &ExpressionType::Bool).map(Self)
    }
    pub fn expression(&self) -> &ApplicationOperationBodyExpression {
        &self.0
    }
    pub fn draft(&self) -> &[u8] {
        self.0.draft()
    }
    pub fn schema(&self) -> &ExpressionSchema {
        self.0.schema()
    }
    pub fn compiled(&self) -> &CompiledExpression {
        self.0.compiled()
    }
    pub fn reads(&self, name: &str) -> bool {
        self.0.reads(name)
    }
    pub(super) fn semantic_material(&self) -> &str {
        self.0.semantic_material()
    }
}
