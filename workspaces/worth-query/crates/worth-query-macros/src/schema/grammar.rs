//! The schema marker's role keys and the keys each role admits.
use super::DERIVE;
use crate::grammar::{flag, type_list, unknown_key, value, visit_keys, Once};
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, LitBool, LitStr, Path, Token, Type};

const KEYS: &[&str] = &[
    "schema",
    "fragment",
    "entity",
    "relation",
    "name",
    "owner",
    "bindings",
    "members",
    "aspects",
    "relations",
    "to",
    "from",
    "reference",
    "cardinality",
    "acyclic",
    "self_edges",
    "unique",
];

pub(super) struct SchemaMarker {
    /// The persisted name; `None` keeps the type name.
    pub(super) name: Option<LitStr>,
    pub(super) role: Role,
}

pub(super) enum Role {
    Root {
        owner: Option<LitStr>,
        bindings: Option<Type>,
        members: Vec<Type>,
    },
    Fragment {
        bindings: Option<Type>,
        members: Vec<Type>,
    },
    Entity {
        aspects: Vec<Type>,
        relations: Vec<Type>,
        reference: Option<Reference>,
    },
    Relation {
        to: Type,
        reference: Option<(Path, Reference)>,
        laws: RelationLaws,
    },
}

pub(super) struct RelationLaws {
    pub(super) cardinality: Option<Expr>,
    pub(super) acyclic: bool,
    pub(super) self_edges: Option<LitBool>,
    pub(super) unique: Option<Expr>,
}

/// `reference(Schema, Fragment)`: where the generated `reference()` points.
pub(super) struct Reference {
    pub(super) schema: Type,
    pub(super) fragment: Path,
}

#[derive(Clone, Copy, PartialEq)]
enum RoleKey {
    Root,
    Fragment,
    Entity,
    Relation,
}

impl RoleKey {
    const fn label(self) -> &'static str {
        match self {
            Self::Root => "a `schema` root",
            Self::Fragment => "a `fragment`",
            Self::Entity => "an `entity`",
            Self::Relation => "a `relation`",
        }
    }
}

struct Keys {
    role: Once<RoleKey>,
    name: Once<LitStr>,
    owner: Once<LitStr>,
    bindings: Once<Type>,
    members: Once<Vec<Type>>,
    aspects: Once<Vec<Type>>,
    relations: Once<Vec<Type>>,
    to: Once<Type>,
    from: Once<Path>,
    reference: Once<Reference>,
    cardinality: Once<Expr>,
    acyclic: Once<()>,
    self_edges: Once<LitBool>,
    unique: Once<Expr>,
}

impl SchemaMarker {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut keys = Keys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let name = std::mem::replace(&mut keys.name, Once::new("name")).optional();
        let role = std::mem::replace(&mut keys.role, Once::new("role"));
        let missing = role.span().unwrap_or(input.ident.span());
        let Some(role) = role.optional() else {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[derive(WorthQuerySchema)] needs one role: \
                 `#[worth_query(schema | fragment | entity | relation, ..)]`",
            ));
        };
        Ok(Self {
            name,
            role: keys.role_for(role, missing)?,
        })
    }
}

impl Keys {
    fn new() -> Self {
        Self {
            role: Once::new("role"),
            name: Once::new("name"),
            owner: Once::new("owner"),
            bindings: Once::new("bindings"),
            members: Once::new("members"),
            aspects: Once::new("aspects"),
            relations: Once::new("relations"),
            to: Once::new("to"),
            from: Once::new("from"),
            reference: Once::new("reference"),
            cardinality: Once::new("cardinality"),
            acyclic: Once::new("acyclic"),
            self_edges: Once::new("self_edges"),
            unique: Once::new("unique"),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        match key {
            "schema" => self.role(meta, RoleKey::Root),
            "fragment" => self.role(meta, RoleKey::Fragment),
            "entity" => self.role(meta, RoleKey::Entity),
            "relation" => self.role(meta, RoleKey::Relation),
            "name" => self.name.set(meta, value(meta)?),
            "owner" => self.owner.set(meta, value(meta)?),
            "bindings" => self.bindings.set(meta, value(meta)?),
            "members" => self.members.set(meta, type_list(meta)?),
            "aspects" => self.aspects.set(meta, type_list(meta)?),
            "relations" => self.relations.set(meta, type_list(meta)?),
            "to" => self.to.set(meta, value(meta)?),
            "from" => self.from.set(meta, value(meta)?),
            "reference" => self.reference.set(meta, reference(meta)?),
            "cardinality" => self.cardinality.set(meta, value(meta)?),
            "acyclic" => {
                flag(meta)?;
                self.acyclic.set(meta, ())
            }
            "self_edges" => self.self_edges.set(meta, value(meta)?),
            "unique" => self.unique.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, KEYS)),
        }
    }

    fn role(&mut self, meta: &ParseNestedMeta<'_>, role: RoleKey) -> syn::Result<()> {
        flag(meta)?;
        if self.role.is_set() {
            return Err(meta.error("a schema marker has exactly one role"));
        }
        self.role.set(meta, role)
    }

    /// Checks that every written key belongs to `role`, then builds it.
    fn role_for(self, role: RoleKey, missing: proc_macro2::Span) -> syn::Result<Role> {
        let label = role.label();
        let rooted = matches!(role, RoleKey::Root | RoleKey::Fragment);
        self.owner.refuse_unless(role == RoleKey::Root, label)?;
        self.bindings.refuse_unless(rooted, label)?;
        self.members.refuse_unless(rooted, label)?;
        self.aspects.refuse_unless(role == RoleKey::Entity, label)?;
        self.relations
            .refuse_unless(role == RoleKey::Entity, label)?;
        self.to.refuse_unless(role == RoleKey::Relation, label)?;
        self.from.refuse_unless(role == RoleKey::Relation, label)?;
        self.cardinality
            .refuse_unless(role == RoleKey::Relation, label)?;
        self.acyclic
            .refuse_unless(role == RoleKey::Relation, label)?;
        self.self_edges
            .refuse_unless(role == RoleKey::Relation, label)?;
        self.unique
            .refuse_unless(role == RoleKey::Relation, label)?;
        let referenced = matches!(role, RoleKey::Entity | RoleKey::Relation);
        self.reference.refuse_unless(referenced, label)?;
        Ok(match role {
            RoleKey::Root => Role::Root {
                owner: self.owner.optional(),
                bindings: self.bindings.optional(),
                members: self.members.required(missing, DERIVE)?,
            },
            RoleKey::Fragment => Role::Fragment {
                bindings: self.bindings.optional(),
                members: self.members.required(missing, DERIVE)?,
            },
            RoleKey::Entity => Role::Entity {
                aspects: self.aspects.required(missing, DERIVE)?,
                relations: self.relations.optional().unwrap_or_default(),
                reference: self.reference.optional(),
            },
            RoleKey::Relation => {
                let to = self.to.required(missing, DERIVE)?;
                let reference_span = self.reference.span();
                let reference = match (self.from.optional(), self.reference.optional()) {
                    (Some(from), Some(reference)) => Some((from, reference)),
                    (None, Some(_)) => {
                        return Err(syn::Error::new(
                            reference_span.unwrap_or(missing),
                            "a relation `reference(..)` also needs `from = Entity`",
                        ));
                    }
                    (_, None) => None,
                };
                Role::Relation {
                    to,
                    reference,
                    laws: RelationLaws {
                        cardinality: self.cardinality.optional(),
                        acyclic: self.acyclic.is_set(),
                        self_edges: self.self_edges.optional(),
                        unique: self.unique.optional(),
                    },
                }
            }
        })
    }
}

fn reference(meta: &ParseNestedMeta<'_>) -> syn::Result<Reference> {
    let content;
    syn::parenthesized!(content in meta.input);
    let schema = content.parse()?;
    content.parse::<Token![,]>()?;
    let fragment = content.parse()?;
    if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
    }
    if !content.is_empty() {
        return Err(content.error("expected `reference(Schema, Fragment)`"));
    }
    Ok(Reference { schema, fragment })
}
