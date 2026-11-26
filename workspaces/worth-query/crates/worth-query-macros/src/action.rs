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
//! block (which may `include: [path::to::fragment];` clauses shared through
//! `worth_query_access_fragment!`), inline capability lists such as
//! `reads(Task)` and an optional
//! `validate = path::to::check`, the input binding's capture-time validator
//! (`fn(&Self) -> Result<(), &'static str>`).
//!
//! Exactly one scope: a field marked `#[worth_query(scope = Selection)]`
//! (`column = ".."` for an override), `principal_scope = Selection`, or
//! `scope = Selection, scope_value = path::to::value` on the input struct, or
//! `scope = Selection, scope_const = expr` for a fixed-duty action whose scope
//! value is the same constant for every input.
//!
//! With `body`, the input binding retains its canonical input
//! (`capture_input`), which authored-body input selectors require. The derive
//! defaults its result to `UnitApplicationValueBinding`, its decision to the
//! body's scoped decision, and its access to a read of the selected scope
//! field. Every named field also receives a compile-time-typed
//! `<field>_input()` accessor (`ApplicationInputField` or
//! `ApplicationOptionalInputField`) for authored body builders. It expands to
//! `worth_query_structured_value_binding!`, `worth_query_operation!`, the
//! contextual `worth_query_mutation_binding!` and, with `delivery`, an
//! `ApplicationActionDelivery` impl. Each identity is the namespace's role
//! identity (`input`, `binding`, `body` or `handler`, `completion`);
//! namespace overrides keep existing wire identities. `member` (or
//! `member = ".."`) narrows a shared namespace to `root.<member>.<role>`, the
//! member defaulting to the snake_case input struct name.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
mod serde_names;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryAction";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = crate::grammar::named_or_unit_fields(input, DERIVE)?;
    let action = grammar::ActionDeclaration::parse(input, fields)?;
    Ok(emit::action(input, &action))
}
