//! The admitted, portable where-filter program: operands plus draft bytes.
use super::{
    ApplicationQueryWhereFilterDenial as Denial, ApplicationQueryWhereFilterOperand as Operand,
};
use std::{cmp::Ordering, sync::Arc};
use worth_foundational::expression_api::{
    expressions, CompiledExpression, ExpressionDraft, ExpressionFunctionCatalog, ExpressionProfile,
    ExpressionSchema, ExpressionType,
};

/// A Boolean Foundation expression admitted once over exact operand types.
/// Equality is the operands, the encoded draft and Foundation's semantic
/// basis; the compiled plan is derived, never a second authority.
#[derive(Clone, Debug)]
pub struct ApplicationQueryWhereFilterProgram {
    operands: Arc<[Operand]>,
    draft: Arc<[u8]>,
    schema: ExpressionSchema,
    compiled: CompiledExpression,
    semantic_material: Arc<str>,
}

impl ApplicationQueryWhereFilterProgram {
    /// The ceilings every where filter is read, admitted and evaluated under.
    pub const PROFILE: ExpressionProfile = ExpressionProfile::interactive();

    /// Readmits encoded draft bytes over `operands` as an untrusted artifact.
    /// This grants no read or execution authority.
    pub fn from_untrusted(operands: Vec<Operand>, draft: &[u8]) -> Result<Self, Denial> {
        let draft = expressions().reading_within(Self::PROFILE).decode(draft)?;
        Self::admit(operands, &draft)
    }

    pub(super) fn admit(
        mut operands: Vec<Operand>,
        draft: &ExpressionDraft,
    ) -> Result<Self, Denial> {
        if operands.is_empty() {
            return Err(Denial::NoOperands);
        }
        if operands
            .iter()
            .any(|operand| operand.name().is_empty() || operand.name().trim() != operand.name())
        {
            return Err(Denial::InvalidOperandName);
        }
        operands.sort_by(|left, right| left.name().cmp(right.name()));
        if operands
            .windows(2)
            .any(|pair| pair[0].name() == pair[1].name())
        {
            return Err(Denial::DuplicateOperand);
        }
        let schema = operands
            .iter()
            .try_fold(ExpressionSchema::builder(), |schema, operand| {
                let ty = operand.source().expression_type()?;
                operand.source().validate_value()?;
                Ok::<_, Denial>(schema.operand(operand.name(), ty)?)
            })?
            .build();
        let catalog = ExpressionFunctionCatalog::builder(&schema, Self::PROFILE).build();
        let admitted = draft.admit_as(&schema, &catalog, Self::PROFILE, &ExpressionType::Bool)?;
        if operands
            .iter()
            .any(|operand| !admitted.slots().any(|(slot, _)| slot == operand.name()))
        {
            return Err(Denial::UnusedOperand);
        }
        let semantic_material = worth_foundational::facade::canonical_basis_sequence_material(
            admitted.identity().basis(),
        )
        .into();
        Ok(Self {
            operands: operands.into(),
            draft: draft.encode().into(),
            compiled: admitted.compile(),
            schema,
            semantic_material,
        })
    }

    /// Operands in name order.
    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }

    pub fn operand(&self, name: &str) -> Option<&Operand> {
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

    pub const fn compiled(&self) -> &CompiledExpression {
        &self.compiled
    }

    /// Foundation's canonical program basis, as stable identity material.
    pub fn semantic_material(&self) -> &str {
        &self.semantic_material
    }

    /// Declaration-owned local check that every parameter operand matches a
    /// declared parameter's scalar family, unitless posture, and optionality.
    pub fn validate_against_parameters(
        &self,
        parameters: &[crate::application_query::ApplicationQueryParameterDefinition],
    ) -> Result<(), crate::application_query::ApplicationQueryDefinitionDenial> {
        super::validation::validate_parameter_operands(parameters, self)
    }

    fn identity_key(&self) -> (&[Operand], &[u8], &str) {
        (&self.operands, &self.draft, &self.semantic_material)
    }
}

impl PartialEq for ApplicationQueryWhereFilterProgram {
    fn eq(&self, other: &Self) -> bool {
        self.identity_key() == other.identity_key()
    }
}
impl Eq for ApplicationQueryWhereFilterProgram {}
impl Ord for ApplicationQueryWhereFilterProgram {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity_key().cmp(&other.identity_key())
    }
}
impl PartialOrd for ApplicationQueryWhereFilterProgram {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
