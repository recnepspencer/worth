//! Struct-level keys, scope selection, and definition defaults for `#[derive(WorthQuery)]`.
use super::fields::parse_fields;
use super::predicates::{
    parse_live_mode, parse_order_by, parse_predicate_list, parse_related_predicate,
    parse_root_path, predicate_key, LiveMode, OrderBySpec, QueryPredicate, QueryRelatedPredicate,
    RootPathSpec,
};
use super::DERIVE;
use crate::case::snake_case;
use crate::grammar::{flag, unknown_key, value, visit_keys, Once};
use crate::identity_member::{parse_member, DeclaredNamespace, RawMember};
use quote::format_ident;
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, Expr, FieldsNamed, Ident, LitStr, Path, Type};

mod definition;
mod scope;
use definition::RawDefinitionKeys;
pub(super) use scope::{QueryScope, StructScope};

const STRUCT_KEYS: &[&str] = &[
    "schema",
    "context",
    "namespace",
    "member",
    "query",
    "input",
    "binding",
    "result",
    "parameters",
    "parameters_fn",
    "bindings",
    "name",
    "scope_name",
    "principal_scope",
    "scope",
    "scope_value",
    "scope_column",
    "limits",
    "validate",
    "root",
    "fields",
    "managed_index",
    "managed_index_refined",
    "cardinality",
    "disclosure",
    "disclose_by",
    "basis_support",
    "lanes",
    "ability",
    "explicit_shape",
    "live",
    "root_path",
    "order_by",
    "where_equal",
    "where_not_equal",
    "where_lt",
    "where_lte",
    "where_gt",
    "where_gte",
    "where_optional",
    "where_equal_optional",
    "where_not_equal_optional",
    "where_lt_optional",
    "where_lte_optional",
    "where_gt_optional",
    "where_gte_optional",
    "where_no_related",
    "where_related_exists",
];

pub(super) struct QueryDeclaration {
    pub(super) schema: Type,
    pub(super) context: Type,
    pub(super) namespace: DeclaredNamespace,
    pub(super) query: Ident,
    pub(super) input: Ident,
    pub(super) binding: Ident,
    pub(super) result: Type,
    pub(super) parameters_binding: Option<Path>,
    pub(super) parameters_fn: Option<Path>,
    pub(super) bindings: Option<Type>,
    pub(super) query_name: LitStr,
    pub(super) scope_name: Option<LitStr>,
    pub(super) limits: Expr,
    pub(super) validate: Option<Path>,
    pub(super) scope: QueryScope,
    pub(super) params: Vec<QueryParam>,
    pub(super) managed_index: Option<ManagedIndexQuery>,
    pub(super) definition: Option<DefinitionDefaults>,
}

pub(super) struct ManagedIndexQuery {
    pub(super) index: Type,
    pub(super) refined: bool,
}

pub(super) struct QueryParam {
    pub(super) member: Ident,
    pub(super) value_ty: Type,
    pub(super) optional_inner: Option<Type>,
    pub(super) wire_name: LitStr,
    pub(super) marker: Ident,
    pub(super) selector: Ident,
    pub(super) binding: Option<Type>,
    pub(super) is_probe: bool,
}

pub(super) struct DefinitionDefaults {
    pub(super) root: Type,
    pub(super) cardinality: Option<Expr>,
    pub(super) disclosure: Option<Expr>,
    pub(super) disclose_by: Option<Expr>,
    pub(super) basis_support: Option<Expr>,
    pub(super) lanes: Option<Expr>,
    pub(super) ability: Option<Expr>,
    pub(super) explicit_shape: bool,
    pub(super) live: Option<LiveMode>,
    pub(super) root_paths: Vec<RootPathSpec>,
    pub(super) predicates: Vec<QueryPredicate>,
    pub(super) related_predicates: Vec<QueryRelatedPredicate>,
    pub(super) order_by: Vec<OrderBySpec>,
}

struct StructKeys {
    schema: Once<Type>,
    context: Once<Type>,
    namespace: Once<Path>,
    member: Once<RawMember>,
    query: Once<Ident>,
    input: Once<Ident>,
    binding: Once<Ident>,
    result: Once<Type>,
    parameters: Once<Path>,
    parameters_fn: Once<Path>,
    bindings: Once<Type>,
    name: Once<LitStr>,
    scope_name: Once<LitStr>,
    principal_scope: Once<Type>,
    scope: Once<Type>,
    scope_value: Once<Path>,
    scope_column: Once<LitStr>,
    limits: Once<Expr>,
    validate: Once<Path>,
    managed_index: Once<Type>,
    managed_index_refined: Once<bool>,
    definition: RawDefinitionKeys,
}

impl QueryDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: Option<&FieldsNamed>) -> syn::Result<Self> {
        let mut keys = StructKeys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let declared_scope = keys.take_scope()?;
        let managed_index = keys.take_managed_index()?;
        let schema =
            std::mem::replace(&mut keys.schema, Once::new("schema")).required(missing, DERIVE)?;
        let parameters_fn =
            std::mem::replace(&mut keys.parameters_fn, Once::new("parameters_fn")).optional();
        if let Some(ref mi) = managed_index
            && parameters_fn.is_some()
        {
            return Err(syn::Error::new_spanned(
                &mi.index,
                "`managed_index` cannot be combined with `parameters_fn`; mark the probe field `#[worth_query(probe)]`",
            ));
        }
        let (scope, params) = parse_fields(
            input,
            fields,
            declared_scope,
            parameters_fn.is_some(),
            managed_index.is_some(),
        )?;
        let definition = keys
            .definition
            .finish(&schema, managed_index.as_ref(), &params)?;
        let query = keys
            .query
            .optional()
            .unwrap_or_else(|| format_ident!("{}Query", input.ident, span = input.ident.span()));
        let input_binding = keys
            .input
            .optional()
            .unwrap_or_else(|| format_ident!("{}Input", input.ident, span = input.ident.span()));
        let binding = keys
            .binding
            .optional()
            .unwrap_or_else(|| format_ident!("{}Binding", input.ident, span = input.ident.span()));
        let result = match (keys.result.optional(), managed_index.as_ref()) {
            (Some(result), _) => result,
            (None, Some(mi)) => {
                let idx = &mi.index;
                syn::parse_quote! {
                    <<#idx as ::worth_query_decl::facade::application_query::ApplicationManagedIndexBinding<#schema>>::SourceBinding
                        as ::worth_query_decl::facade::application_query::ApplicationQueryBinding<#schema>>::ResultBinding
                }
            }
            (None, None) => {
                return Err(syn::Error::new(
                    missing,
                    format!("#[derive({DERIVE})] requires `#[worth_query(result = ..)]`"),
                ));
            }
        };
        let query_name = keys.name.optional().unwrap_or_else(|| {
            LitStr::new(&snake_case(&input.ident.to_string()), input.ident.span())
        });
        Ok(Self {
            schema,
            context: keys.context.required(missing, DERIVE)?,
            namespace: DeclaredNamespace::new(
                keys.namespace.required(missing, DERIVE)?,
                keys.member.optional(),
                &input.ident,
            ),
            query,
            input: input_binding,
            binding,
            result,
            parameters_binding: keys.parameters.optional(),
            parameters_fn,
            bindings: keys.bindings.optional(),
            query_name,
            scope_name: keys.scope_name.optional(),
            limits: keys.limits.required(missing, DERIVE)?,
            validate: keys.validate.optional(),
            scope,
            params,
            managed_index,
            definition,
        })
    }
}

impl StructKeys {
    fn new() -> Self {
        macro_rules! keys {
            ($($field:ident),+ $(,)?) => {
                Self {
                    $($field: Once::new(stringify!($field)),)+
                    definition: RawDefinitionKeys::new(),
                }
            };
        }
        keys!(
            schema,
            context,
            namespace,
            member,
            query,
            input,
            binding,
            result,
            parameters,
            parameters_fn,
            bindings,
            name,
            scope_name,
            principal_scope,
            scope,
            scope_value,
            scope_column,
            limits,
            validate,
            managed_index,
            managed_index_refined,
        )
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        if let Some((op, optional_param)) = predicate_key(key) {
            self.definition
                .predicates
                .extend(parse_predicate_list(meta, op, optional_param)?);
            return Ok(());
        }
        match key {
            "schema" => self.schema.set(meta, value(meta)?),
            "context" => self.context.set(meta, value(meta)?),
            "namespace" => self.namespace.set(meta, value(meta)?),
            "member" => self.member.set(meta, parse_member(meta)?),
            "query" => self.query.set(meta, value(meta)?),
            "input" => self.input.set(meta, value(meta)?),
            "binding" => self.binding.set(meta, value(meta)?),
            "result" => self.result.set(meta, value(meta)?),
            "parameters" => self.parameters.set(meta, value(meta)?),
            "parameters_fn" => self.parameters_fn.set(meta, value(meta)?),
            "bindings" => self.bindings.set(meta, value(meta)?),
            "name" => self.name.set(meta, value(meta)?),
            "scope_name" => self.scope_name.set(meta, value(meta)?),
            "principal_scope" => self.principal_scope.set(meta, value(meta)?),
            "scope" => self.scope.set(meta, value(meta)?),
            "scope_value" => self.scope_value.set(meta, value(meta)?),
            "scope_column" => self.scope_column.set(meta, value(meta)?),
            "limits" => self.limits.set(meta, value(meta)?),
            "validate" => self.validate.set(meta, super::defaults::validator(meta)?),
            "managed_index" => self.managed_index.set(meta, value(meta)?),
            "managed_index_refined" => self.managed_index_refined.set(meta, flag(meta)?),
            "root" => self.definition.root.set(meta, value(meta)?),
            "fields" => self.definition.fields.set(meta, value(meta)?),
            "cardinality" => self.definition.cardinality.set(meta, value(meta)?),
            "disclosure" => self.definition.disclosure.set(meta, value(meta)?),
            "disclose_by" => self.definition.disclose_by.set(meta, value(meta)?),
            "basis_support" => self.definition.basis_support.set(meta, value(meta)?),
            "lanes" => self.definition.lanes.set(meta, value(meta)?),
            "ability" => self.definition.ability.set(meta, value(meta)?),
            "explicit_shape" => self.definition.explicit_shape.set(meta, flag(meta)?),
            "live" => self.definition.live.set(meta, parse_live_mode(meta)?),
            "root_path" => {
                self.definition.root_paths.push(parse_root_path(meta)?);
                Ok(())
            }
            "order_by" => {
                self.definition.order_by.push(parse_order_by(meta)?);
                Ok(())
            }
            "where_no_related" => {
                self.definition
                    .related_predicates
                    .push(parse_related_predicate(meta, false)?);
                Ok(())
            }
            "where_related_exists" => {
                self.definition
                    .related_predicates
                    .push(parse_related_predicate(meta, true)?);
                Ok(())
            }
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        }
    }

    fn take_scope(&mut self) -> syn::Result<Option<StructScope>> {
        scope::resolve_struct_scope(
            std::mem::replace(&mut self.principal_scope, Once::new("principal_scope")),
            std::mem::replace(&mut self.scope, Once::new("scope")),
            std::mem::replace(&mut self.scope_value, Once::new("scope_value")),
            std::mem::replace(&mut self.scope_column, Once::new("scope_column")),
        )
    }

    fn take_managed_index(&mut self) -> syn::Result<Option<ManagedIndexQuery>> {
        let index = std::mem::replace(&mut self.managed_index, Once::new("managed_index"));
        let refined = std::mem::replace(
            &mut self.managed_index_refined,
            Once::new("managed_index_refined"),
        );
        refined.refuse_unless(index.is_set(), "a query without `managed_index = Index`")?;
        Ok(index.optional().map(|index| ManagedIndexQuery {
            index,
            refined: refined.is_set(),
        }))
    }
}
