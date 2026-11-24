//! Role variants, their persisted names and the dependency order they declare.
use super::{DEPENDS_ON, DERIVE};
use crate::grammar::{enum_variants, peeled, unknown_key, value, visit_keys, wire_name, Once};
use std::collections::BTreeSet;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Fields, Ident, LitStr, Token, Type, Variant};

const VARIANT_KEYS: &[&str] = &["name"];

pub(super) struct RoleDeclaration {
    /// Roles in declaration order.
    pub(super) roles: Vec<Role>,
    /// Positions into `roles`, every role after the roles it depends on.
    pub(super) canonical_order: Vec<usize>,
}

pub(super) struct Role {
    pub(super) variant: Ident,
    pub(super) name: LitStr,
    /// Whether the variant carries a numeric index.
    pub(super) indexed: bool,
    /// Positions into the declaration's roles, as written.
    pub(super) depends_on: Vec<Dependency>,
}

pub(super) struct Dependency {
    pub(super) role: usize,
    written: Ident,
}

impl RoleDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let variants = enum_variants(input, DERIVE, "a role")?;
        refuse_keys(&input.attrs, "the role enum; write them on a variant")?;
        let mut roles = variants.iter().map(role).collect::<syn::Result<Vec<_>>>()?;
        let mut names = BTreeSet::new();
        for role in &roles {
            if !names.insert(role.name.value()) {
                return Err(syn::Error::new(
                    role.name.span(),
                    format!("duplicate role `{}`", role.name.value()),
                ));
            }
        }
        for (position, variant) in variants.iter().enumerate() {
            roles[position].depends_on = dependencies(input, &roles, position, variant)?;
        }
        refuse_cycles(&roles)?;
        Ok(Self {
            canonical_order: canonical_order(&roles),
            roles,
        })
    }
}

fn role(variant: &Variant) -> syn::Result<Role> {
    let indexed = match &variant.fields {
        Fields::Unit => false,
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            refuse_keys(&fields.unnamed[0].attrs, "a role's index")?;
            let index = &fields.unnamed[0].ty;
            if !matches!(peeled(index), Type::Path(path) if path.path.is_ident("u32")) {
                return Err(syn::Error::new(
                    index.span(),
                    "a role's index is a `u32`, compared as a number",
                ));
            }
            true
        }
        fields => {
            return Err(syn::Error::new(
                fields.span(),
                "a role variant is a unit, or carries one numeric index such as `Instance(u32)`",
            ));
        }
    };
    let mut name = Once::<LitStr>::new("name");
    visit_keys(&variant.attrs, |key, meta| match key {
        "name" => name.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, VARIANT_KEYS)),
    })?;
    Ok(Role {
        name: wire_name(name.optional(), &variant.ident, "role")?,
        variant: variant.ident.clone(),
        indexed,
        depends_on: Vec::new(),
    })
}

/// Refuses `#[worth_query(..)]` and `#[depends_on(..)]` where the derive
/// would otherwise ignore them.
fn refuse_keys(attributes: &[Attribute], place: &str) -> syn::Result<()> {
    let ignored = attributes.iter().find(|attribute| {
        attribute.path().is_ident(DEPENDS_ON) || attribute.path().is_ident("worth_query")
    });
    match ignored {
        Some(attribute) => Err(syn::Error::new(
            attribute.span(),
            format!("`name` and `{DEPENDS_ON}` do not apply to {place}"),
        )),
        None => Ok(()),
    }
}

/// The roles `variant` names in its `#[depends_on(..)]` attributes.
fn dependencies(
    input: &DeriveInput,
    roles: &[Role],
    position: usize,
    variant: &Variant,
) -> syn::Result<Vec<Dependency>> {
    let mut depends_on = Vec::<Dependency>::new();
    for attribute in variant
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(DEPENDS_ON))
    {
        let written =
            attribute.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)?;
        if written.is_empty() {
            return Err(syn::Error::new(
                attribute.span(),
                "`depends_on` names at least one role variant",
            ));
        }
        for written in written {
            let Some(role) = roles.iter().position(|role| role.variant == written) else {
                let declared: Vec<_> = roles.iter().map(|role| role.variant.to_string()).collect();
                return Err(syn::Error::new(
                    written.span(),
                    format!(
                        "unknown role variant `{written}`; `{}` declares: {}",
                        input.ident,
                        declared.join(", ")
                    ),
                ));
            };
            if role == position {
                return Err(syn::Error::new(
                    written.span(),
                    format!("role `{written}` cannot depend on itself"),
                ));
            }
            if depends_on.iter().any(|dependency| dependency.role == role) {
                return Err(syn::Error::new(
                    written.span(),
                    format!("duplicate dependency `{written}`"),
                ));
            }
            depends_on.push(Dependency { role, written });
        }
    }
    Ok(depends_on)
}

/// Refuses the first dependency that closes a cycle, at its written name.
///
/// Complexity: O(R + D) for R roles and D dependencies.
fn refuse_cycles(roles: &[Role]) -> syn::Result<()> {
    #[derive(Clone, Copy, PartialEq)]
    enum Visit {
        Unvisited,
        Open,
        Closed,
    }
    fn visit(
        roles: &[Role],
        role: usize,
        state: &mut [Visit],
        path: &mut Vec<usize>,
    ) -> syn::Result<()> {
        state[role] = Visit::Open;
        path.push(role);
        for dependency in &roles[role].depends_on {
            match state[dependency.role] {
                Visit::Closed => {}
                Visit::Unvisited => visit(roles, dependency.role, state, path)?,
                Visit::Open => {
                    let start = path.iter().position(|open| *open == dependency.role);
                    let cycle: Vec<_> = path[start.unwrap_or(0)..]
                        .iter()
                        .chain([&dependency.role])
                        .map(|role| roles[*role].variant.to_string())
                        .collect();
                    return Err(syn::Error::new(
                        dependency.written.span(),
                        format!("cyclic role dependencies: {}", cycle.join(" -> ")),
                    ));
                }
            }
        }
        path.pop();
        state[role] = Visit::Closed;
        Ok(())
    }
    let mut state = vec![Visit::Unvisited; roles.len()];
    (0..roles.len()).try_for_each(|role| match state[role] {
        Visit::Unvisited => visit(roles, role, &mut state, &mut Vec::new()),
        _ => Ok(()),
    })
}

/// Every role after the roles it depends on; among roles free to come next,
/// the one declared first. Acyclic roles only.
///
/// Complexity: O(R^2 + R * D), for the handful of roles a scheme declares.
fn canonical_order(roles: &[Role]) -> Vec<usize> {
    let mut order = Vec::with_capacity(roles.len());
    while order.len() < roles.len() {
        let next = (0..roles.len()).find(|role| {
            !order.contains(role)
                && roles[*role]
                    .depends_on
                    .iter()
                    .all(|dependency| order.contains(&dependency.role))
        });
        order.push(next.expect("acyclic roles always leave one free to come next"));
    }
    order
}
