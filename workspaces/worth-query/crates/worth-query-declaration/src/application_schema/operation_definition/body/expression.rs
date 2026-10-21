//! Foundation owns syntax, typing, identity and execution. This binds its slots.
use super::{
    ApplicationOperationBodyRequirementDenial, ApplicationOperationBodyRequirementOperand,
};
use std::{cmp::Ordering, sync::Arc};
use worth_foundational::expression_api::{
    expressions, AdmittedExpression, CompiledExpression, ExpressionProfile, ExpressionSchema,
    ExpressionType,
};

/// A Foundation expression compiled once over exact native body sources.
#[derive(Clone, Debug)]
pub struct ApplicationOperationBodyExpression {
    draft: Arc<[u8]>,
    types: Vec<(String, ExpressionType)>,
    schema: ExpressionSchema,
    admitted: Arc<AdmittedExpression>,
    compiled: CompiledExpression,
    semantic_material: Arc<str>,
    result_type: ExpressionType,
}

impl ApplicationOperationBodyExpression {
    pub const PROFILE: ExpressionProfile = ExpressionProfile::interactive();
    pub(super) fn decode(
        bytes: &[u8],
        operands: &[ApplicationOperationBodyRequirementOperand],
        result_type: &ExpressionType,
    ) -> Result<Self, ApplicationOperationBodyRequirementDenial> {
        let types = operands
            .iter()
            .map(|operand| {
                Ok((
                    operand.name().to_owned(),
                    operand.source().expression_type()?,
                ))
            })
            .collect::<Result<Vec<_>, ApplicationOperationBodyRequirementDenial>>()?;
        let mut builder = ExpressionSchema::builder();
        for operand in operands {
            let ty = if let super::ApplicationOperationBodyRequirementSource::QueryContext {
                query,
                ..
            } = operand.source()
            {
                let (next, ty) = query.result_shape().append_expression_context(builder)?;
                builder = next;
                ty
            } else {
                operand.source().expression_type()?
            };
            builder = builder.operand(operand.name(), ty)?;
        }
        let schema = builder.build();
        let draft = expressions().reading_within(Self::PROFILE).decode(bytes)?;
        let catalog = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(
            &schema,
            Self::PROFILE,
        )
        .build();
        let admitted = draft.admit_as(&schema, &catalog, Self::PROFILE, result_type)?;
        let compiled = admitted.compile();
        let semantic_material = worth_foundational::facade::canonical_basis_sequence_material(
            admitted.identity().basis(),
        )
        .into();
        Ok(Self {
            draft: draft.encode().into(),
            types,
            schema,
            admitted: Arc::new(admitted),
            compiled,
            semantic_material,
            result_type: result_type.clone(),
        })
    }
    pub fn draft(&self) -> &[u8] {
        &self.draft
    }
    pub fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }
    pub fn compiled(&self) -> &CompiledExpression {
        &self.compiled
    }
    pub fn reads(&self, name: &str) -> bool {
        self.admitted.slots().any(|(slot, _)| slot == name)
    }
    pub fn semantic_material(&self) -> &str {
        &self.semantic_material
    }
}

// Exact drafts, typed slots and Foundation's semantic basis retain language and
// profile meaning. Cached evaluation plans are derived, not another authority.
impl PartialEq for ApplicationOperationBodyExpression {
    fn eq(&self, other: &Self) -> bool {
        self.draft == other.draft
            && self.types == other.types
            && self.semantic_material == other.semantic_material
            && self.result_type == other.result_type
    }
}
impl Eq for ApplicationOperationBodyExpression {}
impl Ord for ApplicationOperationBodyExpression {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            &self.draft,
            &self.types,
            &self.semantic_material,
            &self.result_type,
        )
            .cmp(&(
                &other.draft,
                &other.types,
                &other.semantic_material,
                &other.result_type,
            ))
    }
}
impl PartialOrd for ApplicationOperationBodyExpression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
