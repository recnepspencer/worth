//! `#[derive(WorthQuery)]`: query marker, request input binding, parameter
//! references, scope extractor, query binding and optional definition builder
//! from one request struct.
//!
//! Required struct keys: `schema = Schema`, `context = Context`,
//! `namespace = NS`, `query = QueryMarker`, `binding = QueryBinding`,
//! `result = ResultBinding`, `limits = N`.
//!
//! Optional struct keys: `input = InputBinding` (default `<Request>Input`),
//! `name = ".."` (default snake_case of the request struct),
//! `parameters = ParameterBinding` (default `UnitApplicationValueBinding`),
//! `parameters_fn = path::to::fn`, `bindings = BindingSet` (default
//! `NativeScalars`), `scope_name = ".."`, `validate = path::to::check`, and
//! `root = Entity` (emitting `QueryMarker::definition()` and
//! `definition_with(shape)` with defaults `Many`, `Public` disclosure,
//! `CurrentAndPinned` basis, `OneShot` lanes, and `Public` authorization,
//! overridable via `cardinality`, `disclosure`, `basis_support`, `lanes`, and
//! `ability`).
//!
//! Governed shared queries: `disclosure = access::disclosure()` with
//! `disclose_by = access::scope_view()` (both typed, from a stock
//! `worth_query_scope_access!` module) governs the scope key's internal use
//! and every result field and relation slot. `live = effect(Effect, row_key =
//! Row::key(), payload_key = path_or_closure)` declares effect-driven live
//! mode on a root that is its scope: it emits `<Request>LiveCause`, the
//! request's `ApplicationLiveQueryIntent` and the `live_root_by` step.
//!
//! `namespace = NS, member` (or `member = ".."`) shares one namespace among
//! several derives: identities derive as `root.<member>.<role>`, the member
//! defaulting to the snake_case request name.
//!
//! Exactly one scope: a field marked `#[worth_query(scope = Selection)]`
//! (with optional `column = ".."`), struct-level `principal_scope = Selection`,
//! or struct-level `scope = Selection, scope_value = path::to::fn` (with
//! optional `scope_column = ".."`). Every other named field is a typed query
//! parameter (customizable with `param`, `name = ".."`, `binding = ..`,
//! `selector = ..`, or excluded with `skip`).
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod defaults;
mod emit;
mod emit_definition;
mod emit_predicates;
mod fields;
#[cfg(test)]
mod governed_live_tests;
mod grammar;
mod hygiene;
mod live_effect;
mod predicates;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQuery";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = crate::grammar::named_or_unit_fields(input, DERIVE)?;
    let query = grammar::QueryDeclaration::parse(input, fields)?;
    Ok(emit::query(input, &query))
}
