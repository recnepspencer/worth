//! Walk actual field/variant closures with resolved generic substitutions.
use super::modules::Sources;
use super::type_resolution::{builtin_name, type_path, Definition};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use syn::{GenericArgument, GenericParam, Item, PathArguments, Type};

const PLATFORM_REFERENCES: &[&str] = &[
    "EntityId",
    "EntityRef",
    "RecordRef",
    "RelationId",
    "RelationRef",
    "ContentRefId",
    "WorthQueryApplicationEntityKey",
    "WorthQueryApplicationEntityReference",
    "ApplicationEntityKey",
    "ApplicationEntityReference",
];

#[derive(Clone)]
enum Term {
    Builtin(String, Vec<Term>),
    Definition {
        name: String,
        owner: Vec<String>,
        item: Box<Item>,
        arguments: Vec<Term>,
    },
    Array(Box<Term>, String),
    Tuple(Vec<Term>),
}
impl Term {
    fn identity(&self) -> String {
        match self {
            Self::Builtin(name, arguments)
            | Self::Definition {
                name, arguments, ..
            } => format!(
                "{name}<{}>",
                arguments
                    .iter()
                    .map(Self::identity)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Array(element, length) => format!("[{};{length}]", element.identity()),
            Self::Tuple(elements) => format!(
                "({})",
                elements
                    .iter()
                    .map(Self::identity)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

pub(super) fn check(
    sources: &Sources<'_>,
    context: &[String],
    value: &Type,
    forbidden: &[String],
) -> Result<Vec<String>, String> {
    let mut walk = Walk {
        sources,
        forbidden,
        visited: BTreeSet::new(),
        witness: Vec::new(),
        visits: 0,
    };
    let root = walk.resolve(context, value, &BTreeMap::new(), 0)?;
    walk.inspect(&root, 0)?;
    Ok(walk.witness)
}

struct Walk<'a> {
    sources: &'a Sources<'a>,
    forbidden: &'a [String],
    visited: BTreeSet<String>,
    witness: Vec<String>,
    visits: usize,
}
impl Walk<'_> {
    fn admit_visit(&mut self, depth: usize) -> Result<(), String> {
        self.visits += 1;
        if self.visits > 10_000 || depth > 128 {
            return Err("domain type traversal bound exceeded".into());
        }
        Ok(())
    }
    fn deny(&self, full: &str, name: &str) -> Result<(), String> {
        if PLATFORM_REFERENCES.contains(&name)
            || self
                .forbidden
                .iter()
                .any(|value| value == full || value == name)
        {
            return Err(format!(
                "reference/key/name forbidden inside domain value: {full}"
            ));
        }
        Ok(())
    }
    fn resolve(
        &mut self,
        context: &[String],
        value: &Type,
        parameters: &BTreeMap<String, Term>,
        depth: usize,
    ) -> Result<Term, String> {
        self.admit_visit(depth)?;
        match value {
            Type::Array(array) => Ok(Term::Array(
                Box::new(self.resolve(context, &array.elem, parameters, depth + 1)?),
                array.len.to_token_stream().to_string(),
            )),
            Type::Tuple(tuple) => Ok(Term::Tuple(
                tuple
                    .elems
                    .iter()
                    .map(|value| self.resolve(context, value, parameters, depth + 1))
                    .collect::<Result<_, _>>()?,
            )),
            Type::Paren(value) => self.resolve(context, &value.elem, parameters, depth + 1),
            Type::Group(value) => self.resolve(context, &value.elem, parameters, depth + 1),
            Type::Path(path) if path.qself.is_none() => {
                let names = type_path(value)?;
                let last = names.last().ok_or("empty type path")?;
                let arguments = &path.path.segments.last().ok_or("missing type")?.arguments;
                if names.len() == 1 {
                    if let Some(value) = parameters.get(last) {
                        if !matches!(arguments, PathArguments::None) {
                            return Err("generic parameter used as a type constructor".into());
                        }
                        return Ok(value.clone());
                    }
                }
                self.deny(&names.join("::"), last)?;
                if path
                    .path
                    .segments
                    .iter()
                    .take(path.path.segments.len() - 1)
                    .any(|segment| !matches!(segment.arguments, PathArguments::None))
                {
                    return Err("generic module/associated paths are unsupported".into());
                }
                let arguments = match arguments {
                    PathArguments::None => Vec::new(),
                    PathArguments::AngleBracketed(arguments) => arguments.args.iter().map(|argument| match argument {
                        GenericArgument::Type(value) => self.resolve(context, value, parameters, depth + 1),
                        _ => Err("non-type/associated generic arguments require an explicit supported representation".into()),
                    }).collect::<Result<Vec<_>, String>>()?,
                    _ => return Err("function type arguments are unsupported".into()),
                };
                match self.sources.resolve(context, &names, 0)? {
                    Definition::Builtin(name) => {
                        let kind = builtin_name(&name).ok_or("unknown builtin")?.to_owned();
                        if arguments.len()
                            != usize::from(matches!(kind.as_str(), "Vec" | "Box" | "Option"))
                        {
                            return Err(format!("wrong generic arity for {name}"));
                        }
                        Ok(Term::Builtin(kind, arguments))
                    }
                    Definition::Item(item, owner) => {
                        let name = match item.as_ref() {
                            Item::Struct(value) => value.ident.to_string(),
                            Item::Enum(value) => value.ident.to_string(),
                            Item::Type(value) => value.ident.to_string(),
                            _ => {
                                return Err(
                                    "generated/opaque domain definition is unsupported".into()
                                )
                            }
                        };
                        let full = format!("{}::{name}", owner.join("::"));
                        self.deny(&full, &name)?;
                        Ok(Term::Definition {
                            name: full,
                            owner,
                            item,
                            arguments,
                        })
                    }
                }
            }
            _ => Err(format!(
                "unsupported or borrowed domain type: {}",
                value.to_token_stream()
            )),
        }
    }
    fn inspect(&mut self, term: &Term, depth: usize) -> Result<(), String> {
        self.admit_visit(depth)?;
        let identity = term.identity();
        if !self.visited.insert(identity.clone()) {
            return Ok(());
        }
        match term {
            Term::Builtin(_, arguments) | Term::Tuple(arguments) => {
                self.witness.push(identity);
                for value in arguments {
                    self.inspect(value, depth + 1)?;
                }
            }
            Term::Array(value, _) => self.inspect(value, depth + 1)?,
            Term::Definition {
                name,
                owner,
                item,
                arguments,
            } => {
                for argument in arguments {
                    self.inspect(argument, depth + 1)?;
                }
                let (generics, fields): (_, Vec<&Type>) = match item.as_ref() {
                    Item::Struct(value) => (
                        &value.generics,
                        value.fields.iter().map(|field| &field.ty).collect(),
                    ),
                    Item::Enum(value) => (
                        &value.generics,
                        value
                            .variants
                            .iter()
                            .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
                            .collect(),
                    ),
                    Item::Type(value) => (&value.generics, vec![value.ty.as_ref()]),
                    _ => return Err("generated/opaque domain definition is unsupported".into()),
                };
                if generics.params.len() != arguments.len() {
                    return Err(format!(
                        "generic defaults/unknown parameters refused for {name}"
                    ));
                }
                let mut parameters = BTreeMap::new();
                for (parameter, argument) in generics.params.iter().zip(arguments) {
                    let GenericParam::Type(parameter) = parameter else {
                        return Err(
                            "lifetime/const generic definitions require explicit support".into(),
                        );
                    };
                    parameters.insert(parameter.ident.to_string(), argument.clone());
                }
                parameters.insert("Self".into(), term.clone());
                self.witness
                    .push(format!("{identity}: {} actual field edges", fields.len()));
                for field in fields {
                    // Arguments already carry their caller's resolved owner.
                    // Reusing T never resolves a caller name in this callee.
                    let value = self.resolve(owner, field, &parameters, depth + 1)?;
                    self.inspect(&value, depth + 1)?;
                }
            }
        }
        Ok(())
    }
}
