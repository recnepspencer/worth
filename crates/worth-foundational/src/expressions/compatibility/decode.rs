//! Draft decoding: versions first, then bounded, validated structure.
//!
//! Every allocation is bounded by the input, and the input by the profile's
//! `DecodedBytes` ceiling: a count is accepted only when the remaining bytes
//! could hold that many elements at their smallest encoding.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionResource, ExpressionResult, SyntaxDenial,
};
use crate::expressions::profile::{check_limit, ExpressionProfile};
use crate::expressions::syntax::ast::{
    NodeId, QualifiedName, SyntaxNode, SyntaxTree, TypeArgument, TypeSyntax,
};
use crate::expressions::syntax::literal::DecimalText;
use crate::expressions::types::is_identifier;

use super::{tag, BINARY_OPS, COMPREHENSIONS, LANGUAGE_VERSION, MAGIC, UNARY_OPS, WIRE_VERSION};

/// The smallest encodings: a node is a tag and one payload byte, a child a
/// `u32`, a name segment a length.
const MIN_NODE: usize = 2;
const MIN_CHILD: usize = 4;
const MIN_SEGMENT: usize = 4;
const MIN_TYPE_ARGUMENT: usize = 5;
const MIN_FIELD: usize = 8;

pub(crate) fn decode_draft(
    bytes: &[u8],
    profile: &ExpressionProfile,
) -> ExpressionResult<SyntaxTree> {
    check_limit(
        profile,
        ExpressionResource::DecodedBytes,
        bytes.len() as u64,
    )?;
    let mut reader = Reader {
        bytes,
        at: 0,
        type_depth: profile.limit(ExpressionResource::SyntaxDepth),
    };
    if reader.take(MAGIC.len())? != MAGIC {
        return Err(invalid("input is not an expression draft"));
    }
    let wire = reader.u16()?;
    if wire != WIRE_VERSION {
        return Err(unsupported("expression draft encoding", wire));
    }
    let language = reader.u16()?;
    if language != LANGUAGE_VERSION {
        return Err(unsupported("expression language", language));
    }
    let count = reader.u32()? as usize;
    check_limit(profile, ExpressionResource::SyntaxNodes, count as u64)?;
    if count == 0 {
        return Err(invalid("an expression draft has a root node"));
    }
    reader.fits(count, MIN_NODE)?;
    let mut tree = SyntaxTree::new(false);
    let mut used = vec![false; count];
    for index in 0..count {
        let node = reader.node()?;
        for child in node.children() {
            let slot = used
                .get_mut(child.index())
                .filter(|_| child.index() < index)
                .ok_or_else(|| invalid("a child must be an earlier node"))?;
            if std::mem::replace(slot, true) {
                return Err(invalid("a node has exactly one parent"));
            }
        }
        let (_, depth) = tree.push(node, None);
        check_limit(profile, ExpressionResource::SyntaxDepth, u64::from(depth))?;
    }
    if reader.at != bytes.len() {
        return Err(invalid("input continues past the root"));
    }
    if used[..count - 1].contains(&false) {
        return Err(invalid("only the last node may be the root"));
    }
    Ok(tree)
}

fn invalid(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(reason))
}

fn unsupported(artifact: &'static str, found: u16) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::UnsupportedVersion {
        artifact,
        found: u32::from(found),
    })
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Remaining type nesting; type syntax is the only recursive payload.
    type_depth: u64,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> ExpressionResult<&'a [u8]> {
        let end = self
            .at
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid("expression draft is truncated"))?;
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> ExpressionResult<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("take returns N bytes"))
    }

    fn byte(&mut self) -> ExpressionResult<u8> {
        Ok(self.take(1)?[0])
    }

    fn flag(&mut self) -> ExpressionResult<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("a flag is 0 or 1")),
        }
    }

    fn u16(&mut self) -> ExpressionResult<u16> {
        self.array().map(u16::from_le_bytes)
    }

    fn u32(&mut self) -> ExpressionResult<u32> {
        self.array().map(u32::from_le_bytes)
    }

    /// Denies `count` elements the remaining input cannot hold.
    fn fits(&self, count: usize, smallest: usize) -> ExpressionResult<()> {
        match count.checked_mul(smallest) {
            Some(needed) if needed <= self.bytes.len() - self.at => Ok(()),
            _ => Err(invalid("expression draft is truncated")),
        }
    }

    fn count(&mut self, smallest: usize) -> ExpressionResult<usize> {
        let count = self.u32()? as usize;
        self.fits(count, smallest)?;
        Ok(count)
    }

    fn child(&mut self) -> ExpressionResult<NodeId> {
        self.u32().map(NodeId)
    }

    fn children(&mut self) -> ExpressionResult<Vec<NodeId>> {
        let count = self.count(MIN_CHILD)?;
        (0..count).map(|_| self.child()).collect()
    }

    fn text(&mut self) -> ExpressionResult<Box<str>> {
        let length = self.u32()? as usize;
        let bytes = self.take(length)?;
        std::str::from_utf8(bytes)
            .map(Box::from)
            .map_err(|_| invalid("text is not UTF-8"))
    }

    fn identifier(&mut self) -> ExpressionResult<Box<str>> {
        let text = self.text()?;
        if is_identifier(&text) {
            Ok(text)
        } else {
            Err(ExpressionDenial::new(ExpressionDenialDetail::Syntax(
                SyntaxDenial::InvalidIdentifier,
            )))
        }
    }

    fn name(&mut self) -> ExpressionResult<QualifiedName> {
        let count = self.count(MIN_SEGMENT)?;
        if count == 0 {
            return Err(invalid("a name has at least one segment"));
        }
        (0..count)
            .map(|_| self.identifier())
            .collect::<Result<_, _>>()
            .map(QualifiedName)
    }

    fn operator<T: Copy>(&mut self, table: &[T]) -> ExpressionResult<T> {
        let code = self.byte()?;
        table
            .get(usize::from(code))
            .copied()
            .ok_or_else(|| invalid("unknown operator code"))
    }

    fn float(&mut self) -> ExpressionResult<DecimalText> {
        let digits = self.text()?;
        let exponent = i32::from_le_bytes(self.array()?);
        // Only the normalized spelling decodes, so equal drafts encode equally.
        DecimalText::parse(&format!("{digits}e{exponent}"))
            .filter(|decimal| decimal.digits() == &*digits && decimal.exponent() == exponent)
            .ok_or_else(|| invalid("float literal is not normalized decimal text"))
    }

    fn node(&mut self) -> ExpressionResult<SyntaxNode> {
        Ok(match self.byte()? {
            tag::BOOL => SyntaxNode::Bool(self.flag()?),
            tag::INTEGER => SyntaxNode::Integer(u128::from_le_bytes(self.array()?)),
            tag::FLOAT => SyntaxNode::Float(self.float()?),
            tag::STRING => SyntaxNode::String(self.text()?),
            tag::NAME => SyntaxNode::Name(self.name()?),
            tag::FIELD => SyntaxNode::Field {
                base: self.child()?,
                field: self.identifier()?,
            },
            tag::COMPREHENSION => SyntaxNode::Comprehension {
                base: self.child()?,
                kind: self.operator(&COMPREHENSIONS)?,
                binder: self.identifier()?,
                body: self.child()?,
            },
            tag::UNARY => SyntaxNode::Unary {
                op: self.operator(&UNARY_OPS)?,
                operand: self.child()?,
            },
            tag::BINARY => SyntaxNode::Binary {
                op: self.operator(&BINARY_OPS)?,
                left: self.child()?,
                right: self.child()?,
            },
            tag::CONDITIONAL => SyntaxNode::Conditional {
                condition: self.child()?,
                then: self.child()?,
                otherwise: self.child()?,
            },
            tag::LET => SyntaxNode::Let {
                binder: self.identifier()?,
                value: self.child()?,
                body: self.child()?,
            },
            tag::CALL => SyntaxNode::Call {
                function: self.name()?,
                type_arguments: self.type_arguments()?,
                arguments: self.children()?,
            },
            tag::NONE => SyntaxNode::None(self.type_syntax()?),
            tag::LIST => SyntaxNode::List(self.children()?),
            tag::MAP => {
                let count = self.count(2 * MIN_CHILD)?;
                let entries = (0..count)
                    .map(|_| Ok((self.child()?, self.child()?)))
                    .collect::<ExpressionResult<_>>()?;
                SyntaxNode::Map(entries)
            }
            tag::RECORD => {
                let type_name = self.name()?;
                let count = self.count(MIN_FIELD)?;
                let fields = (0..count)
                    .map(|_| Ok((self.identifier()?, self.child()?)))
                    .collect::<ExpressionResult<_>>()?;
                SyntaxNode::Record { type_name, fields }
            }
            _ => return Err(invalid("unknown syntax node tag")),
        })
    }

    fn type_syntax(&mut self) -> ExpressionResult<TypeSyntax> {
        self.type_depth = self
            .type_depth
            .checked_sub(1)
            .ok_or_else(|| invalid("type syntax nests too deeply"))?;
        let name = self.name()?;
        let version = if self.flag()? {
            Some(self.u32()?)
        } else {
            None
        };
        let arguments = self.type_arguments()?;
        self.type_depth += 1;
        Ok(TypeSyntax {
            name,
            arguments,
            version,
        })
    }

    fn type_arguments(&mut self) -> ExpressionResult<Vec<TypeArgument>> {
        let count = self.count(MIN_TYPE_ARGUMENT)?;
        (0..count).map(|_| self.type_argument()).collect()
    }

    fn type_argument(&mut self) -> ExpressionResult<TypeArgument> {
        Ok(match self.byte()? {
            tag::TYPE => TypeArgument::Type(self.type_syntax()?),
            tag::WIDTH => TypeArgument::Width(self.u32()?),
            tag::PRODUCT => {
                let count = self.count(1 + MIN_SEGMENT)?;
                let factors = (0..count)
                    .map(|_| Ok((self.flag()?, self.name()?)))
                    .collect::<ExpressionResult<_>>()?;
                TypeArgument::Product(factors)
            }
            _ => return Err(invalid("unknown type argument tag")),
        })
    }
}
