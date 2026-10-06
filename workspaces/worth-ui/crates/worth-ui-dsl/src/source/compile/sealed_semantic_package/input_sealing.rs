use crate::source::{
    WorthUiArtifactInputAppearanceRoleNode, WorthUiArtifactInputBackdropNode,
    WorthUiArtifactInputBlockNode, WorthUiArtifactInputImportNode, WorthUiArtifactInputLayoutNode,
    WorthUiArtifactInputNode, WorthUiArtifactInputSemanticArtifactNode,
    WorthUiArtifactInputTokenNode,
};
use crate::WorthUiExpressionIntroducer;

/// How one input node reaches the sealed package. Expression declarations
/// need every projection and every other expression, so they wait for the
/// package-level admission pass; every other declaration seals on its own.
pub(super) enum InputSealing<'a> {
    Declaration(SealableInput<'a>),
    Expression(ExpressionInput<'a>),
}

/// An expression declaration held for package-level admission.
pub(super) struct ExpressionInput<'a> {
    pub(super) block: &'a WorthUiArtifactInputBlockNode,
    pub(super) introducer: WorthUiExpressionIntroducer,
}

/// An input node that seals on its own. It has no expression variants, so
/// per-declaration sealing cannot be handed a deferred declaration.
pub(super) enum SealableInput<'a> {
    Import(&'a WorthUiArtifactInputImportNode),
    Component(&'a WorthUiArtifactInputBlockNode),
    Surface(&'a WorthUiArtifactInputBlockNode),
    Binding(&'a WorthUiArtifactInputBlockNode),
    QueryScalar(&'a WorthUiArtifactInputBlockNode),
    QueryCollection(&'a WorthUiArtifactInputBlockNode),
    Token(&'a WorthUiArtifactInputTokenNode),
    SemanticArtifact(&'a WorthUiArtifactInputSemanticArtifactNode),
    AppearanceRole(&'a WorthUiArtifactInputAppearanceRoleNode),
    Backdrop(&'a WorthUiArtifactInputBackdropNode),
    Layout(&'a WorthUiArtifactInputLayoutNode),
}

impl<'a> InputSealing<'a> {
    pub(super) fn classify(node: &'a WorthUiArtifactInputNode) -> Self {
        match node {
            WorthUiArtifactInputNode::Import(node) => {
                Self::Declaration(SealableInput::Import(node))
            }
            WorthUiArtifactInputNode::Component(node) => {
                Self::Declaration(SealableInput::Component(node))
            }
            WorthUiArtifactInputNode::Surface(node) => {
                Self::Declaration(SealableInput::Surface(node))
            }
            WorthUiArtifactInputNode::Binding(node) => {
                Self::Declaration(SealableInput::Binding(node))
            }
            WorthUiArtifactInputNode::QueryScalar(node) => {
                Self::Declaration(SealableInput::QueryScalar(node))
            }
            WorthUiArtifactInputNode::QueryCollection(node) => {
                Self::Declaration(SealableInput::QueryCollection(node))
            }
            WorthUiArtifactInputNode::Token(node) => Self::Declaration(SealableInput::Token(node)),
            WorthUiArtifactInputNode::SemanticArtifact(node) => {
                Self::Declaration(SealableInput::SemanticArtifact(node))
            }
            WorthUiArtifactInputNode::AppearanceRole(node) => {
                Self::Declaration(SealableInput::AppearanceRole(node))
            }
            WorthUiArtifactInputNode::Backdrop(node) => {
                Self::Declaration(SealableInput::Backdrop(node))
            }
            WorthUiArtifactInputNode::Layout(node) => {
                Self::Declaration(SealableInput::Layout(node))
            }
            WorthUiArtifactInputNode::Condition(block) => Self::Expression(ExpressionInput {
                block,
                introducer: WorthUiExpressionIntroducer::When,
            }),
            WorthUiArtifactInputNode::Derived(block) => Self::Expression(ExpressionInput {
                block,
                introducer: WorthUiExpressionIntroducer::Value,
            }),
        }
    }
}
