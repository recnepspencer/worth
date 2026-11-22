/// Declares a validated identity namespace in const context.
///
/// Input: `"root.v1"`, then optional explicit overrides `Role => "wire.id"`
/// for identities that already exist on the wire. Each step is the fallible
/// `ApplicationIdentityNamespace` constructor; a refusal fails compilation
/// with the denial's message.
///
/// ```
/// use worth_query_declaration::{worth_query_identity, worth_query_identity_namespace};
/// use worth_query_declaration::facade::application_identity::ApplicationIdentityNamespace;
/// const ADD: ApplicationIdentityNamespace = worth_query_identity_namespace!(
///     "daymark.tasks.add.v1", Binding => "daymark.add.binding.v1"
/// );
/// const INPUT: &str = worth_query_identity!(ADD, input);
/// const BINDING: &str = worth_query_identity!(ADD, binding);
/// const TITLE: &str = worth_query_identity!(ADD, slot "title");
/// assert_eq!(INPUT, "daymark.tasks.add.v1.input");
/// assert_eq!(BINDING, "daymark.add.binding.v1");
/// assert_eq!(TITLE, "daymark.tasks.add.v1.slot.title");
/// ```
///
/// An override may not reuse another role's identity:
///
/// ```compile_fail,E0080
/// use worth_query_declaration::worth_query_identity_namespace;
/// use worth_query_declaration::facade::application_identity::ApplicationIdentityNamespace;
/// const ADD: ApplicationIdentityNamespace = worth_query_identity_namespace!(
///     "daymark.tasks.add.v1", Binding => "daymark.tasks.add.v1.input"
/// );
/// ```
#[macro_export]
macro_rules! worth_query_identity_namespace {
    ($root:expr $(, $Role:ident => $identity:expr)* $(,)?) => {
        const {
            let namespace = match $crate::facade::application_identity::ApplicationIdentityNamespace::new($root) {
                ::core::result::Result::Ok(namespace) => namespace,
                ::core::result::Result::Err(denial) => ::core::panic!("{}", denial.message()),
            };
            $(
                let namespace = match namespace.with_override(
                    $crate::facade::application_identity::ApplicationIdentityRole::$Role,
                    $identity,
                ) {
                    ::core::result::Result::Ok(namespace) => namespace,
                    ::core::result::Result::Err(denial) => ::core::panic!("{}", denial.message()),
                };
            )*
            namespace
        }
    };
}

/// Evaluates one namespace identity to a `&'static str` at compile time.
///
/// Input: `NAMESPACE, role` (`input`, `binding`, `request`, `result`, ...) or
/// `NAMESPACE, slot "field"`. The result fits every `identity` position of
/// the binding, value and query declaration macros. An invalid slot fails
/// compilation with the denial's message.
#[macro_export]
macro_rules! worth_query_identity {
    ($namespace:expr, slot $field:expr) => {
        $crate::worth_query_identity!(@text match $namespace.slot($field) {
            ::core::result::Result::Ok(identity) => identity,
            ::core::result::Result::Err(denial) => ::core::panic!("{}", denial.message()),
        })
    };
    ($namespace:expr, $role:ident) => {
        $crate::worth_query_identity!(@text $namespace.$role())
    };
    (@text $identity:expr) => {{
        const IDENTITY: $crate::facade::application_identity::ApplicationDerivedIdentity = $identity;
        const BYTES: [u8; IDENTITY.len()] = IDENTITY.to_bytes();
        const TEXT: &str = match ::core::str::from_utf8(&BYTES) {
            ::core::result::Result::Ok(text) => text,
            ::core::result::Result::Err(_) => ::core::panic!("derived identities are ASCII"),
        };
        TEXT
    }};
}
