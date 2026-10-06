//! Draft encoding. Encoding is total: every draft has exactly one encoding.

use crate::expressions::syntax::ast::{
    QualifiedName, SyntaxNode, SyntaxTree, TypeArgument, TypeSyntax,
};

use super::{
    code, tag, BINARY_OPS, COMPREHENSIONS, LANGUAGE_VERSION, MAGIC, UNARY_OPS, WIRE_VERSION,
};

pub(crate) fn encode_draft(tree: &SyntaxTree) -> Vec<u8> {
    let mut out = Writer(Vec::new());
    out.0.extend_from_slice(&MAGIC);
    out.0.extend_from_slice(&WIRE_VERSION.to_le_bytes());
    out.0.extend_from_slice(&LANGUAGE_VERSION.to_le_bytes());
    out.count(tree.len());
    for node in tree.nodes() {
        out.node(node);
    }
    out.0
}

struct Writer(Vec<u8>);

impl Writer {
    fn byte(&mut self, value: u8) {
        self.0.push(value);
    }

    fn count(&mut self, value: usize) {
        self.0.extend_from_slice(&(value as u32).to_le_bytes());
    }

    fn child(&mut self, id: crate::expressions::syntax::ast::NodeId) {
        self.count(id.index());
    }

    fn text(&mut self, value: &str) {
        self.count(value.len());
        self.0.extend_from_slice(value.as_bytes());
    }

    fn name(&mut self, name: &QualifiedName) {
        self.count(name.0.len());
        name.0.iter().for_each(|segment| self.text(segment));
    }

    fn node(&mut self, node: &SyntaxNode) {
        match node {
            SyntaxNode::Bool(value) => {
                self.byte(tag::BOOL);
                self.byte(u8::from(*value));
            }
            SyntaxNode::Integer(value) => {
                self.byte(tag::INTEGER);
                self.0.extend_from_slice(&value.to_le_bytes());
            }
            SyntaxNode::Float(decimal) => {
                self.byte(tag::FLOAT);
                self.text(decimal.digits());
                self.0.extend_from_slice(&decimal.exponent().to_le_bytes());
            }
            SyntaxNode::String(value) => {
                self.byte(tag::STRING);
                self.text(value);
            }
            SyntaxNode::Name(name) => {
                self.byte(tag::NAME);
                self.name(name);
            }
            SyntaxNode::Field { base, field } => {
                self.byte(tag::FIELD);
                self.child(*base);
                self.text(field);
            }
            SyntaxNode::Comprehension {
                base,
                kind,
                binder,
                body,
            } => {
                self.byte(tag::COMPREHENSION);
                self.child(*base);
                self.byte(code(&COMPREHENSIONS, kind));
                self.text(binder);
                self.child(*body);
            }
            SyntaxNode::Unary { op, operand } => {
                self.byte(tag::UNARY);
                self.byte(code(&UNARY_OPS, op));
                self.child(*operand);
            }
            SyntaxNode::Binary { op, left, right } => {
                self.byte(tag::BINARY);
                self.byte(code(&BINARY_OPS, op));
                self.child(*left);
                self.child(*right);
            }
            SyntaxNode::Conditional {
                condition,
                then,
                otherwise,
            } => {
                self.byte(tag::CONDITIONAL);
                [condition, then, otherwise]
                    .into_iter()
                    .for_each(|child| self.child(*child));
            }
            SyntaxNode::Let {
                binder,
                value,
                body,
            } => {
                self.byte(tag::LET);
                self.text(binder);
                self.child(*value);
                self.child(*body);
            }
            SyntaxNode::Call {
                function,
                type_arguments,
                arguments,
            } => {
                self.byte(tag::CALL);
                self.name(function);
                self.count(type_arguments.len());
                type_arguments
                    .iter()
                    .for_each(|argument| self.type_argument(argument));
                self.count(arguments.len());
                arguments.iter().for_each(|argument| self.child(*argument));
            }
            SyntaxNode::None(ty) => {
                self.byte(tag::NONE);
                self.type_syntax(ty);
            }
            SyntaxNode::List(items) => {
                self.byte(tag::LIST);
                self.count(items.len());
                items.iter().for_each(|item| self.child(*item));
            }
            SyntaxNode::Map(entries) => {
                self.byte(tag::MAP);
                self.count(entries.len());
                for (key, value) in entries {
                    self.child(*key);
                    self.child(*value);
                }
            }
            SyntaxNode::Record { type_name, fields } => {
                self.byte(tag::RECORD);
                self.name(type_name);
                self.count(fields.len());
                for (field, value) in fields {
                    self.text(field);
                    self.child(*value);
                }
            }
        }
    }

    fn type_syntax(&mut self, ty: &TypeSyntax) {
        self.name(&ty.name);
        match ty.version {
            Some(version) => {
                self.byte(1);
                self.0.extend_from_slice(&version.to_le_bytes());
            }
            None => self.byte(0),
        }
        self.count(ty.arguments.len());
        ty.arguments
            .iter()
            .for_each(|argument| self.type_argument(argument));
    }

    fn type_argument(&mut self, argument: &TypeArgument) {
        match argument {
            TypeArgument::Type(ty) => {
                self.byte(tag::TYPE);
                self.type_syntax(ty);
            }
            TypeArgument::Width(width) => {
                self.byte(tag::WIDTH);
                self.0.extend_from_slice(&width.to_le_bytes());
            }
            TypeArgument::Product(factors) => {
                self.byte(tag::PRODUCT);
                self.count(factors.len());
                for (divide, name) in factors {
                    self.byte(u8::from(*divide));
                    self.name(name);
                }
            }
        }
    }
}
