use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use worth_foundational::expression_api::{
    AdmittedExpression, ExpressionProgramIdentity, ExpressionSchema,
};

use super::{
    WorthUiSealedExpressionOperand, WorthUiSealedSemanticPackage, WorthUiSemanticProvenanceRef,
};
use crate::source::WorthUiDslSourceSpan;
use crate::WorthUiExpressionRole;

/// A `condition` or `derived` declaration after shared-kernel admission.
///
/// Equality and hashing cover the declaration identity, role, operands, and
/// the kernel program identity. They never cover spans, provenance, or the
/// admitted program's pointer, so equal meaning is equal however it was
/// authored.
#[derive(Clone, Debug)]
pub struct WorthUiSealedExpression {
    identity: String,
    role: WorthUiExpressionRole,
    operands: Box<[WorthUiSealedExpressionOperand]>,
    schema: ExpressionSchema,
    draft: Arc<[u8]>,
    admitted: Arc<AdmittedExpression>,
    provenance_ref: WorthUiSemanticProvenanceRef,
    body_span: Option<WorthUiDslSourceSpan>,
}

impl WorthUiSealedExpression {
    pub(super) fn new(
        identity: String,
        role: WorthUiExpressionRole,
        operands: Box<[WorthUiSealedExpressionOperand]>,
        schema: ExpressionSchema,
        draft: Vec<u8>,
        admitted: AdmittedExpression,
        provenance_ref: WorthUiSemanticProvenanceRef,
        body_span: Option<WorthUiDslSourceSpan>,
    ) -> Self {
        Self {
            identity,
            role,
            operands,
            schema,
            draft: draft.into(),
            admitted: Arc::new(admitted),
            provenance_ref,
            body_span,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn role(&self) -> WorthUiExpressionRole {
        self.role
    }

    /// Operands in name order, with their resolved kernel types.
    pub fn operands(&self) -> &[WorthUiSealedExpressionOperand] {
        &self.operands
    }

    /// The operand types evaluation binds values against.
    pub fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }

    /// The versioned kernel draft encoding, without source spans.
    pub fn draft(&self) -> &[u8] {
        &self.draft
    }

    pub fn admitted(&self) -> &Arc<AdmittedExpression> {
        &self.admitted
    }

    /// The canonical meaning of the admitted program.
    pub fn program_identity(&self) -> &ExpressionProgramIdentity {
        self.admitted.identity()
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }

    /// Where the expression text sits in its host file. Rust-authored
    /// declarations have no host file and report `None`.
    pub fn body_span(&self) -> Option<&WorthUiDslSourceSpan> {
        self.body_span.as_ref()
    }
}

impl PartialEq for WorthUiSealedExpression {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.role == other.role
            && self.operands == other.operands
            && self.program_identity() == other.program_identity()
    }
}

impl Eq for WorthUiSealedExpression {}

impl Hash for WorthUiSealedExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
        self.role.hash(state);
        self.operands.hash(state);
        self.program_identity().digest().value().bytes().hash(state);
    }
}

impl WorthUiSealedSemanticPackage {
    /// Every sealed expression in identity order.
    pub fn expressions(&self) -> impl ExactSizeIterator<Item = &WorthUiSealedExpression> {
        self.expressions.values()
    }

    pub fn expression(&self, identity: &str) -> Option<&WorthUiSealedExpression> {
        self.expressions.get(identity)
    }
}
