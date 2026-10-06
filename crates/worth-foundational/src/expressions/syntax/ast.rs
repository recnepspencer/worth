//! Arena syntax tree shared by source, builder, and decoded drafts.
//!
//! Children always precede their parent, so every pass can walk the arena in
//! index order without recursion, and tree depth is recorded per node.

use super::literal::DecimalText;
use super::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeId(pub(crate) u32);

impl NodeId {
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A `::`-separated name. Segments are ASCII identifiers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct QualifiedName(pub(crate) Vec<Box<str>>);

impl QualifiedName {
    pub(crate) fn single(&self) -> Option<&str> {
        match self.0.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> String {
        self.0.join("::")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOp {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Coalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ComprehensionKind {
    Map,
    Filter,
    All,
    Any,
}

/// Authored type syntax. It resolves only against the admitted schema.
///
/// Source text names nominal types without a version; a Rust builder records
/// the exact version it meant, and resolution denies a different one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypeSyntax {
    pub(crate) name: QualifiedName,
    pub(crate) arguments: Vec<TypeArgument>,
    pub(crate) version: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TypeArgument {
    Type(TypeSyntax),
    Width(u32),
    /// `length * length / time`: each factor is a dimension name.
    Product(Vec<(bool, QualifiedName)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SyntaxNode {
    Bool(bool),
    Integer(u128),
    Float(DecimalText),
    String(Box<str>),
    Name(QualifiedName),
    Field {
        base: NodeId,
        field: Box<str>,
    },
    Comprehension {
        base: NodeId,
        kind: ComprehensionKind,
        binder: Box<str>,
        body: NodeId,
    },
    Unary {
        op: UnaryOp,
        operand: NodeId,
    },
    Binary {
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
    },
    Conditional {
        condition: NodeId,
        then: NodeId,
        otherwise: NodeId,
    },
    Let {
        binder: Box<str>,
        value: NodeId,
        body: NodeId,
    },
    Call {
        function: QualifiedName,
        type_arguments: Vec<TypeArgument>,
        arguments: Vec<NodeId>,
    },
    None(TypeSyntax),
    List(Vec<NodeId>),
    Map(Vec<(NodeId, NodeId)>),
    Record {
        type_name: QualifiedName,
        fields: Vec<(Box<str>, NodeId)>,
    },
}

impl SyntaxNode {
    /// Children in authored evaluation order.
    pub(crate) fn children(&self) -> Vec<NodeId> {
        match self {
            Self::Bool(_) | Self::Integer(_) | Self::Float(_) | Self::String(_) => Vec::new(),
            Self::Name(_) | Self::None(_) => Vec::new(),
            Self::Field { base, .. } => vec![*base],
            Self::Comprehension { base, body, .. } => vec![*base, *body],
            Self::Unary { operand, .. } => vec![*operand],
            Self::Binary { left, right, .. } => vec![*left, *right],
            Self::Conditional {
                condition,
                then,
                otherwise,
            } => vec![*condition, *then, *otherwise],
            Self::Let { value, body, .. } => vec![*value, *body],
            Self::Call { arguments, .. } | Self::List(arguments) => arguments.clone(),
            Self::Map(entries) => entries.iter().flat_map(|(k, v)| [*k, *v]).collect(),
            Self::Record { fields, .. } => fields.iter().map(|(_, value)| *value).collect(),
        }
    }

    /// A copy with every child replaced by `map(child)`, in `children` order.
    pub(crate) fn map_children(&self, mut map: impl FnMut(NodeId) -> NodeId) -> Self {
        let mut node = self.clone();
        match &mut node {
            Self::Bool(_) | Self::Integer(_) | Self::Float(_) | Self::String(_) => {}
            Self::Name(_) | Self::None(_) => {}
            Self::Field { base, .. } | Self::Unary { operand: base, .. } => *base = map(*base),
            Self::Comprehension { base, body, .. }
            | Self::Let {
                value: base, body, ..
            } => {
                *base = map(*base);
                *body = map(*body);
            }
            Self::Binary { left, right, .. } => {
                *left = map(*left);
                *right = map(*right);
            }
            Self::Conditional {
                condition,
                then,
                otherwise,
            } => {
                for child in [condition, then, otherwise] {
                    *child = map(*child);
                }
            }
            Self::Call { arguments, .. } | Self::List(arguments) => {
                arguments.iter_mut().for_each(|child| *child = map(*child));
            }
            Self::Map(entries) => {
                for (key, value) in entries {
                    *key = map(*key);
                    *value = map(*value);
                }
            }
            Self::Record { fields, .. } => {
                fields
                    .iter_mut()
                    .for_each(|(_, value)| *value = map(*value));
            }
        }
        node
    }
}

/// A syntax arena with a single root. Nodes are in post-order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxTree {
    nodes: Vec<SyntaxNode>,
    depths: Vec<u16>,
    spans: Option<Vec<SourceSpan>>,
}

impl SyntaxTree {
    pub(crate) fn new(with_spans: bool) -> Self {
        Self {
            nodes: Vec::new(),
            depths: Vec::new(),
            spans: with_spans.then(Vec::new),
        }
    }

    /// Appends a node whose children are already present, returning its id
    /// and depth. Callers enforce node-count and depth ceilings.
    pub(crate) fn push(&mut self, node: SyntaxNode, span: Option<SourceSpan>) -> (NodeId, u16) {
        let depth = node
            .children()
            .iter()
            .map(|child| self.depths[child.index()])
            .max()
            .map_or(1, |deepest| deepest.saturating_add(1));
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        self.depths.push(depth);
        if let (Some(spans), Some(span)) = (self.spans.as_mut(), span) {
            spans.push(span);
        }
        (id, depth)
    }

    pub(crate) fn node(&self, id: NodeId) -> &SyntaxNode {
        &self.nodes[id.index()]
    }

    pub(crate) fn nodes(&self) -> &[SyntaxNode] {
        &self.nodes
    }

    pub(crate) fn depth(&self, id: NodeId) -> u16 {
        self.depths[id.index()]
    }

    pub(crate) fn span(&self, id: NodeId) -> Option<SourceSpan> {
        self.spans
            .as_ref()
            .and_then(|spans| spans.get(id.index()).copied())
    }

    pub(crate) fn root(&self) -> Option<NodeId> {
        self.nodes
            .len()
            .checked_sub(1)
            .map(|last| NodeId(last as u32))
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}
