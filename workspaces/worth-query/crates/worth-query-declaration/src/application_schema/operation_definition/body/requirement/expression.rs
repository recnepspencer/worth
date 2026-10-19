//! Foundation owns syntax, typing, identity and execution. This binds its slots.
use super::{
    ApplicationOperationBodyRequirementDenial, ApplicationOperationBodyRequirementOperand,
};
use std::{cmp::Ordering, sync::Arc};
use worth_foundational::expression_api::{
    expressions, AdmittedExpression, CompiledExpression, ExpressionProfile, ExpressionSchema,
    ExpressionType,
};

/// A Boolean Foundation expression compiled once over the requirement's sources.
#[derive(Clone, Debug)]
pub struct ApplicationOperationBodyRequirementExpression {
    draft: Arc<[u8]>,
    types: Vec<(String, ExpressionType)>,
    schema: ExpressionSchema,
    admitted: Arc<AdmittedExpression>,
    compiled: CompiledExpression,
    semantic_material: Arc<str>,
}

impl ApplicationOperationBodyRequirementExpression {
    pub const PROFILE: ExpressionProfile = ExpressionProfile::interactive();
    pub(super) fn decode(
        bytes: &[u8],
        operands: &[ApplicationOperationBodyRequirementOperand],
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
        let schema = types
            .iter()
            .try_fold(ExpressionSchema::builder(), |schema, (name, ty)| {
                schema.operand(name, ty.clone())
            })?
            .build();
        let draft = expressions().reading_within(Self::PROFILE).decode(bytes)?;
        let catalog = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(
            &schema,
            Self::PROFILE,
        )
        .build();
        let admitted = draft.admit_as(&schema, &catalog, Self::PROFILE, &ExpressionType::Bool)?;
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
    pub(super) fn semantic_material(&self) -> &str {
        &self.semantic_material
    }
}

// Exact drafts, typed slots and Foundation's semantic basis retain language and
// profile meaning. Cached evaluation plans are derived, not another authority.
impl PartialEq for ApplicationOperationBodyRequirementExpression {
    fn eq(&self, other: &Self) -> bool {
        self.draft == other.draft
            && self.types == other.types
            && self.semantic_material == other.semantic_material
    }
}
impl Eq for ApplicationOperationBodyRequirementExpression {}
impl Ord for ApplicationOperationBodyRequirementExpression {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.draft, &self.types, &self.semantic_material).cmp(&(
            &other.draft,
            &other.types,
            &other.semantic_material,
        ))
    }
}
impl PartialOrd for ApplicationOperationBodyRequirementExpression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
