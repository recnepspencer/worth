use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::expression_api::{
    CompiledExpression, ExpressionProgramIdentity, ExpressionSchema,
};
use worth_ui_dsl::{WorthUiDslSourceSpan, WorthUiExpressionRole, WorthUiSealedExpression};
use worth_ui_query_binding::WorthUiQueryBindingPlan;

use super::operand_resolution::UiExpressionOperandResolver;
use super::slot::UiExpressionSlotCount;
use super::topological_order::topological_order;
use super::{
    UiExpressionCatalogPreparationDenial, UiExpressionDependencyIndex, UiExpressionSlot,
    UiResolvedExpressionOperand, WorthUiAuthoredExpressionMaterial,
};
use crate::declaration::UiIntentApplicationFactPlan;
use crate::runtime::WorthUiAuthoredProjectionRequirement;

/// Every sealed expression of one prepared generation, installed once.
#[derive(Debug)]
pub(crate) struct UiExpressionCatalog {
    slot_count: UiExpressionSlotCount,
    expressions: Box<[Arc<UiInstalledExpression>]>,
    slots_by_identity: BTreeMap<Box<str>, UiExpressionSlot>,
    dependencies: UiExpressionDependencyIndex,
}

#[derive(Debug)]
pub(crate) struct UiInstalledExpression {
    identity: Box<str>,
    role: WorthUiExpressionRole,
    operands: Box<[UiInstalledExpressionOperand]>,
    schema: ExpressionSchema,
    program: CompiledExpression,
    program_identity: ExpressionProgramIdentity,
    body_span: Option<Arc<WorthUiDslSourceSpan>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiInstalledExpressionOperand {
    name: Box<str>,
    source: UiResolvedExpressionOperand,
}

impl UiExpressionCatalog {
    pub(crate) fn prepare(
        material: &WorthUiAuthoredExpressionMaterial,
        projections: &[WorthUiAuthoredProjectionRequirement],
        query: &WorthUiQueryBindingPlan,
        application_facts: &UiIntentApplicationFactPlan,
    ) -> Result<Self, UiExpressionCatalogPreparationDenial> {
        let sealed = material.expressions();
        let slot_count = UiExpressionSlotCount::of(sealed.len())?;
        let order = topological_order(sealed)?;
        let expression_slots: BTreeMap<&str, UiExpressionSlot> = order
            .iter()
            .zip(slot_count.slots())
            .map(|(&index, slot)| (sealed[index].identity(), slot))
            .collect();
        let resolver = UiExpressionOperandResolver {
            projections,
            query,
            application_facts,
            expression_slots: &expression_slots,
        };
        let expressions = order
            .iter()
            .map(|&index| UiInstalledExpression::install(&sealed[index], &resolver).map(Arc::new))
            .collect::<Result<Box<[_]>, _>>()?;
        let slots_by_identity = expression_slots
            .into_iter()
            .map(|(identity, slot)| (Box::from(identity), slot))
            .collect();
        let dependencies = UiExpressionDependencyIndex::build(
            &expressions,
            slot_count,
            application_facts.entries().len(),
        );
        Ok(Self {
            slot_count,
            expressions,
            slots_by_identity,
            dependencies,
        })
    }

    /// The slots of this catalog's expressions.
    pub(crate) const fn slot_count(&self) -> UiExpressionSlotCount {
        self.slot_count
    }

    pub(crate) fn expression(&self, slot: UiExpressionSlot) -> Option<&Arc<UiInstalledExpression>> {
        self.expressions.get(slot.index())
    }

    pub(crate) fn slot_of(&self, identity: &str) -> Option<UiExpressionSlot> {
        self.slots_by_identity.get(identity).copied()
    }

    pub(crate) fn dependencies(&self) -> &UiExpressionDependencyIndex {
        &self.dependencies
    }

    /// Whether `successor` installs the same expressions in the same slots.
    pub(crate) fn has_same_slots_as(&self, successor: &Self) -> bool {
        self.slot_count == successor.slot_count
            && self.slots_by_identity == successor.slots_by_identity
    }

    /// Whether `successor` installs the same program in every slot, so every
    /// result it would retain means what this catalog's does. Spans are
    /// evidence and take no part.
    pub(crate) fn installs_same_expressions_as(&self, successor: &Self) -> bool {
        self.has_same_slots_as(successor)
            && self.slot_count.slots().all(|slot| {
                match (self.expression(slot), successor.expression(slot)) {
                    (Some(prior), Some(next)) => prior.installs_same_program_as(next),
                    (None, None) => true,
                    _ => false,
                }
            })
    }
}

impl UiInstalledExpression {
    fn install(
        sealed: &WorthUiSealedExpression,
        resolver: &UiExpressionOperandResolver<'_>,
    ) -> Result<Self, UiExpressionCatalogPreparationDenial> {
        let operands = sealed
            .operands()
            .iter()
            .map(|operand| {
                Ok(UiInstalledExpressionOperand {
                    name: operand.name().into(),
                    source: resolver.resolve(sealed.identity(), operand)?,
                })
            })
            .collect::<Result<_, UiExpressionCatalogPreparationDenial>>()?;
        Ok(Self {
            identity: sealed.identity().into(),
            role: sealed.role(),
            operands,
            schema: sealed.schema().clone(),
            program: sealed.admitted().compile(),
            program_identity: sealed.program_identity().clone(),
            body_span: sealed.body_span().cloned().map(Arc::new),
        })
    }

    /// Whether `successor` runs the same program over the same operands, so
    /// a result of this expression is a result of `successor`. The body span
    /// is evidence only and takes no part.
    pub(crate) fn installs_same_program_as(&self, successor: &Self) -> bool {
        self.identity == successor.identity
            && self.role == successor.role
            && self.operands == successor.operands
            && self.program_identity == successor.program_identity
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn role(&self) -> WorthUiExpressionRole {
        self.role
    }

    /// Operands in name order.
    pub(crate) fn operands(&self) -> &[UiInstalledExpressionOperand] {
        &self.operands
    }

    pub(crate) fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }

    pub(crate) fn program(&self) -> &CompiledExpression {
        &self.program
    }

    pub(crate) fn program_identity(&self) -> &ExpressionProgramIdentity {
        &self.program_identity
    }

    pub(crate) fn body_span(&self) -> Option<&Arc<WorthUiDslSourceSpan>> {
        self.body_span.as_ref()
    }
}

impl UiInstalledExpressionOperand {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn source(&self) -> &UiResolvedExpressionOperand {
        &self.source
    }
}
