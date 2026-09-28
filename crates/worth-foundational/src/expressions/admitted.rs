//! Admission of a top-level expression into sealed, typed, canonical meaning.

use std::sync::Arc;

use super::admission::{check_tree, AdmissionContext, Scope};
use super::canonical::{derive_identity, ExpressionProgramIdentity, IdentityInputs};
use super::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use super::draft::ExpressionDraft;
use super::functions::InstalledExpressionFunction;
use super::profile::ExpressionProfile;
use super::program::ExpressionProgram;
use super::syntax::ExpressionSourceMap;
use super::types::{ExpressionSchema, ExpressionType};

/// A checked expression: resolved slots, types, function closure, and
/// canonical identity. Only [`ExpressionDraft::admit`] constructs one; it
/// carries structural and type facts, never world or runtime authority.
#[derive(Debug, Clone)]
pub struct AdmittedExpression {
    program: ExpressionProgram,
    slots: Box<[(Box<str>, ExpressionType)]>,
    closure: Box<[Arc<InstalledExpressionFunction>]>,
    identity: ExpressionProgramIdentity,
    source_map: ExpressionSourceMap,
    /// The type declarations operand values are checked against.
    declarations: ExpressionSchema,
    profile: ExpressionProfile,
    work: u64,
    expanded_instructions: u64,
    call_depth: u32,
}

/// Admits `draft`: resolves every name, checks every type and branch,
/// resolves overloads and the function closure, checks resource ceilings, and
/// derives canonical identity.
pub(crate) fn admit(
    draft: &ExpressionDraft,
    context: AdmissionContext<'_>,
    expected: Option<&ExpressionType>,
) -> ExpressionResult<AdmittedExpression> {
    let AdmissionContext {
        schema,
        catalog,
        profile,
    } = context;
    draft.check_within(profile)?;
    if !schema.same_declarations(catalog.schema()) {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::UnsupportedFeature(
                "the function catalog was installed over different type declarations",
            ),
        ));
    }
    if let Some(expected) = expected {
        schema.check_type(expected)?;
    }
    let checked = check_tree(draft.tree(), &context, Scope::Operands, expected)?;
    let closure: Box<[_]> = checked
        .functions
        .iter()
        .map(|index| catalog.function(*index).clone())
        .collect();
    let identity = derive_identity(IdentityInputs {
        program: &checked.program,
        slots: checked
            .slots
            .iter()
            .map(|(name, ty)| (Some(&**name), ty))
            .collect(),
        closure: &closure,
        schema,
        profile,
    })?;
    let source_map = ExpressionSourceMap::new(
        checked
            .program
            .nodes()
            .iter()
            .map(|node| node.origin)
            .collect(),
    );
    Ok(AdmittedExpression {
        program: checked.program,
        slots: checked.slots.into_boxed_slice(),
        closure,
        identity,
        source_map,
        declarations: schema.declarations_only(),
        profile: *profile,
        work: checked.work,
        expanded_instructions: checked.expanded_instructions,
        call_depth: checked.call_depth,
    })
}

impl AdmittedExpression {
    pub fn result_type(&self) -> &ExpressionType {
        self.program.result_type()
    }

    /// The operands the expression reads, in canonical name order. Slot
    /// indices in the program refer to this order.
    pub fn slots(&self) -> impl ExactSizeIterator<Item = (&str, &ExpressionType)> {
        self.slots.iter().map(|(name, ty)| (&**name, ty))
    }

    pub fn identity(&self) -> &ExpressionProgramIdentity {
        &self.identity
    }

    /// Canonical node origins in the authored draft.
    pub fn source_map(&self) -> &ExpressionSourceMap {
        &self.source_map
    }

    /// Direct installed callees in canonical closure order.
    pub fn functions(&self) -> impl ExactSizeIterator<Item = &Arc<InstalledExpressionFunction>> {
        self.closure.iter()
    }

    /// Admission work units spent checking this expression.
    pub fn admission_work(&self) -> u64 {
        self.work
    }

    /// Own instructions plus every call site's expanded callee instructions.
    pub fn expanded_instructions(&self) -> u64 {
        self.expanded_instructions
    }

    pub fn call_depth(&self) -> u32 {
        self.call_depth
    }

    /// The profile the expression was admitted under; evaluation never
    /// exceeds it.
    pub fn profile(&self) -> &ExpressionProfile {
        &self.profile
    }

    pub(crate) fn program(&self) -> &ExpressionProgram {
        &self.program
    }

    pub(crate) fn declarations(&self) -> &ExpressionSchema {
        &self.declarations
    }
}
