//! Numeric parsing follows explicit Rust types or a direct expected type.
//! Unresolved inference is refused instead of assuming a numeric width.
use crate::rust_identifier::semantic_name;
use syn::{visit_mut::VisitMut, Expr, GenericArgument, PathArguments, Type};

#[derive(Clone, Copy)]
pub(super) enum Integer {
    U8,
    U32,
}

pub(super) fn parse_type(call: &syn::ExprMethodCall) -> Result<Integer, String> {
    let args = call
        .turbofish
        .as_ref()
        .ok_or("generator numeric parse type is unresolved")?;
    if args.args.len() == 1
        && let Some(GenericArgument::Type(ty)) = args.args.first()
        && let Some(kind) = integer(ty)
    {
        return Ok(kind);
    }
    Err("generator numeric parse type requires qualification".into())
}

fn integer(ty: &Type) -> Option<Integer> {
    let Type::Path(path) = ty else { return None };
    match semantic_name(path.path.get_ident()?).as_str() {
        "u8" => Some(Integer::U8),
        "u32" => Some(Integer::U32),
        _ => None,
    }
}

#[derive(Clone, Copy, Default)]
enum Hint {
    #[default]
    Unknown,
    Integer(Integer),
    Optional(Integer),
    Result(Integer),
}

fn hint(ty: &Type) -> Hint {
    if let Some(kind) = integer(ty) {
        return Hint::Integer(kind);
    }
    if let Type::Path(path) = ty
        && path.path.segments.len() == 1
        && let Some(segment) = path.path.segments.first()
        && semantic_name(&segment.ident) == "Option"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let Some(GenericArgument::Type(ty)) = args.args.first()
        && let Some(kind) = integer(ty)
    {
        return Hint::Optional(kind);
    }
    Hint::Unknown
}

pub(super) fn resolve(function: &mut syn::ItemFn) -> Result<(), String> {
    let returned = match &function.sig.output {
        syn::ReturnType::Type(_, ty) => hint(ty),
        _ => Hint::Unknown,
    };
    let mut resolver = Resolver {
        expected: returned,
        returned,
        error: None,
    };
    resolver.visit_block_mut(&mut function.block);
    resolver.error.map_or(Ok(()), Err)
}

struct Resolver {
    expected: Hint,
    returned: Hint,
    error: Option<String>,
}
impl Resolver {
    fn expression(&mut self, expr: &mut Expr, expected: Hint) {
        self.expected = expected;
        self.visit_expr_mut(expr);
    }
}
impl VisitMut for Resolver {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let expected = std::mem::take(&mut self.expected);
        let last = block.stmts.len().saturating_sub(1);
        for (index, statement) in block.stmts.iter_mut().enumerate() {
            self.expected = if index == last {
                expected
            } else {
                Hint::Unknown
            };
            self.visit_stmt_mut(statement);
        }
    }
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        let expected = match &local.pat {
            syn::Pat::Type(ty) => hint(&ty.ty),
            _ => Hint::Unknown,
        };
        if let Some(init) = &mut local.init {
            self.expression(&mut init.expr, expected);
            if let Some((_, diverge)) = &mut init.diverge {
                self.expression(diverge, Hint::Unknown);
            }
        }
    }
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        let expected = std::mem::take(&mut self.expected);
        match expr {
            Expr::MethodCall(call) => {
                let method = semantic_name(&call.method);
                if method == "parse" {
                    if call.turbofish.is_none()
                        && let Hint::Result(kind) = expected
                    {
                        call.turbofish = Some(match kind {
                            Integer::U8 => syn::parse_quote!(::<u8>),
                            Integer::U32 => syn::parse_quote!(::<u32>),
                        });
                    }
                    if let Err(error) = parse_type(call) {
                        self.error.get_or_insert(error);
                    }
                }
                let receiver = match (method.as_str(), expected) {
                    ("unwrap", Hint::Integer(kind)) | ("ok", Hint::Optional(kind)) => {
                        Hint::Result(kind)
                    }
                    _ => Hint::Unknown,
                };
                self.expression(&mut call.receiver, receiver);
                for arg in &mut call.args {
                    self.expression(arg, Hint::Unknown);
                }
            }
            Expr::Paren(value) => self.expression(&mut value.expr, expected),
            Expr::Block(value) => {
                self.expected = expected;
                self.visit_block_mut(&mut value.block);
            }
            Expr::Return(value) => {
                if let Some(expr) = &mut value.expr {
                    self.expression(expr, self.returned);
                }
            }
            Expr::If(value) => {
                self.expression(&mut value.cond, Hint::Unknown);
                self.expected = expected;
                self.visit_block_mut(&mut value.then_branch);
                if let Some((_, other)) = &mut value.else_branch {
                    self.expression(other, expected);
                }
            }
            Expr::Match(value) => {
                self.expression(&mut value.expr, Hint::Unknown);
                for arm in &mut value.arms {
                    if let Some((_, guard)) = &mut arm.guard {
                        self.expression(guard, Hint::Unknown);
                    }
                    self.expression(&mut arm.body, expected);
                }
            }
            _ => syn::visit_mut::visit_expr_mut(self, expr),
        }
    }
}
