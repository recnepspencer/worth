//! Selects optional callbacks for automatically registered providers.
use super::IndexProviderKind;
use syn::Expr;

pub(super) fn provider_refinement(kind: &IndexProviderKind, expr: &Expr) -> syn::Result<bool> {
    match expr {
        Expr::Paren(paren) => return provider_refinement(kind, &paren.expr),
        Expr::Group(group) => return provider_refinement(kind, &group.expr),
        Expr::Path(path)
            if path
                .path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "None") =>
        {
            return Ok(false);
        }
        Expr::Call(call)
            if matches!(&*call.func, Expr::Path(path)
                if path.path.segments.last().is_some_and(|seg| seg.ident == "Some")) =>
        {
            if matches!(kind, IndexProviderKind::Custom { .. }) {
                return Ok(true);
            }
            return Err(syn::Error::new_spanned(
                expr,
                "stock managed index kinds do not support refinement; use `kind = custom(provider)` with a refinement-capable provider",
            ));
        }
        _ => {}
    }
    Err(syn::Error::new_spanned(
        expr,
        "automatic provider registration requires `refinement = Some(..)` or `refinement = None`",
    ))
}
