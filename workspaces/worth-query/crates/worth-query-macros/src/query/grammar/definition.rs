//! Definition-defaults validation and assembly for `#[derive(WorthQuery)]`.
use super::{DefinitionDefaults, ManagedIndexQuery, QueryParam};
use crate::grammar::Once;
use crate::query::predicates::{
    bare_expr_ident, resolve_predicates, resolve_related_predicates, LiveMode, OrderBySpec,
    RawPredicate, RawRelatedPredicate, RootPathSpec,
};
use syn::{Expr, Path, Type};

pub(super) struct RawDefinitionKeys {
    pub(super) root: Once<Type>,
    pub(super) fields: Once<Path>,
    pub(super) cardinality: Once<Expr>,
    pub(super) disclosure: Once<Expr>,
    pub(super) basis_support: Once<Expr>,
    pub(super) lanes: Once<Expr>,
    pub(super) ability: Once<Expr>,
    pub(super) explicit_shape: Once<bool>,
    pub(super) live: Once<LiveMode>,
    pub(super) root_paths: Vec<RootPathSpec>,
    pub(super) predicates: Vec<RawPredicate>,
    pub(super) related_predicates: Vec<RawRelatedPredicate>,
    pub(super) order_by: Vec<OrderBySpec>,
}

impl RawDefinitionKeys {
    pub(super) fn new() -> Self {
        Self {
            root: Once::new("root"),
            fields: Once::new("fields"),
            cardinality: Once::new("cardinality"),
            disclosure: Once::new("disclosure"),
            basis_support: Once::new("basis_support"),
            lanes: Once::new("lanes"),
            ability: Once::new("ability"),
            explicit_shape: Once::new("explicit_shape"),
            live: Once::new("live"),
            root_paths: Vec::new(),
            predicates: Vec::new(),
            related_predicates: Vec::new(),
            order_by: Vec::new(),
        }
    }

    pub(super) fn finish(
        self,
        schema: &Type,
        managed_index: Option<&ManagedIndexQuery>,
        params: &[QueryParam],
    ) -> syn::Result<Option<DefinitionDefaults>> {
        let has_root = self.root.is_set() || managed_index.is_some();
        let label = "a query without `root = Entity`";
        self.cardinality.refuse_unless(has_root, label)?;
        self.disclosure.refuse_unless(has_root, label)?;
        self.basis_support.refuse_unless(has_root, label)?;
        self.lanes.refuse_unless(has_root, label)?;
        self.ability.refuse_unless(has_root, label)?;
        self.explicit_shape.refuse_unless(has_root, label)?;
        self.fields.refuse_unless(has_root, label)?;
        self.live.refuse_unless(has_root, label)?;
        if !has_root {
            refuse_first_span(&self.root_paths, |x| x.span, "`root_path`", label)?;
            refuse_first_span(&self.predicates, |x| x.span, "`where_*` predicates", label)?;
            refuse_first_span(
                &self.related_predicates,
                |x| x.span,
                "related predicates",
                label,
            )?;
            refuse_first_span(&self.order_by, |x| x.span, "`order_by`", label)?;
        }
        if managed_index.is_some() {
            let mi_label = "a query with `managed_index`";
            self.live.refuse_unless(false, mi_label)?;
            refuse_first_span(&self.root_paths, |x| x.span, "`root_path`", mi_label)?;
            refuse_first_span(
                &self.predicates,
                |x| x.span,
                "`where_*` predicates",
                mi_label,
            )?;
            refuse_first_span(
                &self.related_predicates,
                |x| x.span,
                "related predicates",
                mi_label,
            )?;
        }
        let live = self.live.optional();
        let lanes = self.lanes.optional();
        if live == Some(LiveMode::WholeResult)
            && let Some(ref expr) = lanes
            && bare_expr_ident(expr).is_some_and(|ident| ident == "OneShot")
        {
            return Err(syn::Error::new_spanned(
                expr,
                "`live = whole_result` requires live lane eligibility; omit `lanes` or use `lanes = OneShotAndLive`",
            ));
        }

        let resolved_root = match (self.root.optional(), managed_index) {
            (Some(root), _) => Some(root),
            (None, Some(mi)) => Some(crate::query::defaults::managed_index_root_type(
                schema, &mi.index,
            )),
            (None, None) => None,
        };
        let fields_path = self.fields.optional();
        let predicates = resolve_predicates(self.predicates, fields_path.as_ref(), params)?;
        let related_predicates =
            resolve_related_predicates(self.related_predicates, fields_path.as_ref(), params)?;
        Ok(resolved_root.map(|root| DefinitionDefaults {
            root,
            cardinality: self.cardinality.optional(),
            disclosure: self.disclosure.optional(),
            basis_support: self.basis_support.optional(),
            lanes,
            ability: self.ability.optional(),
            explicit_shape: self.explicit_shape.is_set(),
            live,
            root_paths: self.root_paths,
            predicates,
            related_predicates,
            order_by: self.order_by,
        }))
    }
}

fn refuse_first_span<T>(
    items: &[T],
    span: impl Fn(&T) -> proc_macro2::Span,
    name: &str,
    context: &str,
) -> syn::Result<()> {
    if let Some(first) = items.first() {
        return Err(syn::Error::new(
            span(first),
            format!("{name} does not apply to {context}"),
        ));
    }
    Ok(())
}
