//! Named access fragments that operation `access` blocks include by path.
//!
//! A fragment is a declarative macro holding `kind: [..];` access clauses.
//! An `include: [fragment, ..];` clause expands each fragment in place, so the
//! including operation's `access` block lowers exactly as if the clauses were
//! written inline: per-operation Rust capability impls and schema projection.

/// Declares a reusable, named list of operation access clauses.
///
/// Input: `name { clauses }`, where each clause is one of the `access` kinds
/// `worth_query_operation!` accepts (`read_entities`, `read_fields`,
/// `read_relations`, `writes`, `expects_version`, `expects_fact`, `creates`,
/// `deletes`, `links`, `unlinks`, `emits`), each written `kind: [..];`, or
/// `include: [other_fragment, ..];`. Attributes such as doc comments apply to
/// the generated macro. It generates a declarative macro `name` that an
/// `access { include: [name]; .. }` block (or `#[derive(WorthQueryAction)]`'s
/// `access(include: [name]; ..)`) expands in place.
///
/// Reach a fragment from a sibling module by re-exporting it after its
/// definition (`pub(crate) use name;`) and including it by path
/// (`include: [super::posting::name]`). Paths written *inside* a fragment,
/// members and nested includes alike, resolve where the fragment is included,
/// not where it is declared: name them `crate::`-qualified, or import them in
/// every including module.
///
/// An operation's declared access is a set. Field-valued members
/// (`read_fields`, `writes`, `expects_version`, `expects_fact`) may repeat
/// across fragments and inline clauses; the schema registers each once.
/// Entity, relation and effect members (`read_entities`, `read_relations`,
/// `creates`, `deletes`, `links`, `unlinks`, `emits`) each implement a Rust
/// capability trait for the including operation, so one operation names each
/// of those members once across everything it includes; a repeat is a
/// conflicting-impl compile error.
///
/// ```
/// use worth_query_declaration::facade::application_schema::*;
/// use worth_query_declaration::{worth_query_access_fragment, worth_query_operation,
///     worth_query_structured_value_binding};
/// struct Schema;
/// pub struct Bin;
/// impl fragment::Entity for Bin {const NAME:&'static str="Bin";type Aspects=();type Relations=();}
/// pub struct Movement;
/// impl fragment::Entity for Movement {const NAME:&'static str="Movement";type Aspects=();type Relations=();}
/// mod posting {
///     worth_query_declaration::worth_query_access_fragment! {
///         /// Every stock posting reads a bin.
///         stock_posting_access { read_entities: [crate::Bin]; }
///     }
///     pub(crate) use stock_posting_access;
/// }
/// worth_query_structured_value_binding!(Input for () { identity: "example.fragment.input.v1" });
/// worth_query_operation!(Post for Schema, input Input;
///     access { include: [posting::stock_posting_access]; creates: [Movement]; }
/// );
/// fn reads<T: OperationReads<Post>>() {}
/// fn creates<T: OperationCreates<Post>>() {}
/// fn main() {
///     reads::<Bin>();
///     creates::<Movement>();
/// }
/// ```
///
/// Naming an entity both in an included fragment and inline is refused:
///
/// ```compile_fail,E0119
/// use worth_query_declaration::facade::application_schema::*;
/// use worth_query_declaration::{worth_query_access_fragment, worth_query_operation,
///     worth_query_structured_value_binding};
/// struct Schema;
/// pub struct Bin;
/// impl fragment::Entity for Bin {const NAME:&'static str="Bin";type Aspects=();type Relations=();}
/// worth_query_access_fragment! { bins { read_entities: [Bin]; } }
/// worth_query_structured_value_binding!(Input for () { identity: "example.fragment.input.v1" });
/// worth_query_operation!(Post for Schema, input Input;
///     access { include: [bins]; read_entities: [Bin]; }
/// );
/// ```
#[macro_export]
macro_rules! worth_query_access_fragment {
    ($(#[$attribute:meta])* $Fragment:ident { $($clauses:tt)* }) => {
        $crate::__worth_query_access_fragment_definition!(
            ($) $(#[$attribute])* $Fragment { $($clauses)* }
        );
    };
}

/// Emits one fragment macro. `$d` carries a literal `$` so the generated macro
/// can declare its own repetition over the include continuation.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_access_fragment_definition {
    (($d:tt) $(#[$attribute:meta])* $Fragment:ident { $($clauses:tt)* }) => {
        $(#[$attribute])*
        macro_rules! $Fragment {
            (@worth_query_access_include $d($d continuation:tt)*) => {
                $crate::__worth_query_operation_access_flatten!(
                    @fragment { $($clauses)* } $d($d continuation)*
                );
            };
        }
    };
}

/// Flattens an `access` block's includes into its plain `kind: [..];` clauses,
/// then lowers them through `__worth_query_operation_access!`.
///
/// State: `[flattened clauses] { remaining clauses } (lowering head) (lowering
/// tail)`. A remaining `include: [first, rest..];` calls `first!`, which
/// prepends its own clauses and `include: [rest..];` to the remaining ones.
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_operation_access_flatten {
    (@fragment { $($fragment:tt)* } [$($flat:tt)*] { $($remaining:tt)* } $($lowering:tt)*) => {
        $crate::__worth_query_operation_access_flatten!(
            [$($flat)*] { $($fragment)* $($remaining)* } $($lowering)*
        );
    };
    ([$($flat:tt)*] { include: [$(,)?]; $($remaining:tt)* } $($lowering:tt)*) => {
        $crate::__worth_query_operation_access_flatten!(
            [$($flat)*] { $($remaining)* } $($lowering)*
        );
    };
    ([$($flat:tt)*] { include: [$first:path $(, $rest:path)* $(,)?]; $($remaining:tt)* }
        $($lowering:tt)*
    ) => {
        $first!(@worth_query_access_include
            [$($flat)*] { include: [$($rest),*]; $($remaining)* } $($lowering)*
        );
    };
    ([$($flat:tt)*] { include $($malformed:tt)* } $($lowering:tt)*) => {
        ::core::compile_error!(
            "`include` takes access fragment macro paths, such as `include: [posting::stock_access];`"
        );
    };
    ([$($flat:tt)*] { $kind:ident: [$($member:tt)*]; $($remaining:tt)* } $($lowering:tt)*) => {
        $crate::__worth_query_operation_access_flatten!(
            [$($flat)* $kind: [$($member)*];] { $($remaining)* } $($lowering)*
        );
    };
    ([$($flat:tt)*] {} ($($head:tt)*) ($($tail:tt)*)) => {
        $crate::__worth_query_operation_access!($($head)* { $($flat)* } $($tail)*);
    };
}
