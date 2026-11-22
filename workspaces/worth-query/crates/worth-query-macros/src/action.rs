//! `#[derive(WorthQueryAction)]`: one mutation action from its input struct.
//!
//! Struct keys: `schema = Schema`, `context = Context` (an
//! `ApplicationBindingContext` supplying principal, idempotency and denial),
//! `namespace = NAMESPACE`, the generated type names `input = InputBinding`,
//! `operation = Operation` and `binding = Binding`, plus an optional
//! `delivery = Delivery` (a source-completion delivery), `result =
//! ResultBinding`, `decision = Decision`, optional `outputs = Outputs`, the
//! flags `body` (the handler is the operation's authored body) and `program`
//! (execution requires the application program), the forwarded `access(..)`
//! block and inline capability lists such as `reads(Task)`.
//!
//! Exactly one scope: a field marked `#[worth_query(scope = Selection)]`
//! (`column = ".."` when the column is named otherwise) or the struct key
//! `principal_scope = Selection`.
//!
//! With `body`, the input binding retains its canonical input
//! (`capture_input`), which authored-body input selectors require. It expands to
//! `worth_query_structured_value_binding!`,
//! `worth_query_operation!`, the contextual `worth_query_mutation_binding!`
//! and, with `delivery`, an `ApplicationActionDelivery` impl. Each identity is
//! the namespace's role identity (`input`, `binding`, `body` or `handler`,
//! `completion`); namespace overrides keep existing wire identities.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryAction";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = crate::grammar::named_fields(input, DERIVE)?;
    let action = grammar::ActionDeclaration::parse(input, fields)?;
    Ok(emit::action(input, &action))
}
