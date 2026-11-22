//! Standalone compile-time capability declarations of one operation.
//!
//! `worth_query_operation!` accepts the same families inline
//! (`reads [..]; creates [..]; links [..];`) and expands to these macros.

/// Declares, at compile time, that an operation may write the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationWrites<Operation>` for each listed marker. This is descriptive;
/// installed operation authority is still required.
#[macro_export]
macro_rules! worth_query_operation_writes {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationWrites<$Operation> for $Field {}
        )+
    };
}

/// Declares, at compile time, that an operation may read the listed schema
/// members.
///
/// Input: `Operation => [Member, ..]`. It implements
/// `OperationReads<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_reads {
    ($Operation:ty => [$($Member:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationReads<$Operation> for $Member {}
        )+
    };
}

/// Declares, at compile time, that an operation may carry an expected-version
/// precondition on the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationExpectsVersion<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_expects_version {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationExpectsVersion<$Operation>
                for $Field
            {}
        )+
    };
}

/// Declares, at compile time, that an operation may carry an expected-fact
/// precondition on the listed fields.
///
/// Input: `Operation => [Field, ..]`. It implements
/// `OperationExpectsFact<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_expects_fact {
    ($Operation:ty => [$($Field:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationExpectsFact<$Operation>
                for $Field
            {}
        )+
    };
}

/// Declares, at compile time, that an operation may create the listed entities.
///
/// Input: `Operation => [Entity, ..]`. It implements
/// `OperationCreates<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_creates {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationCreates<$Operation> for $Entity {}
        )+
    };
}

/// Declares, at compile time, that an operation may delete the listed entities.
///
/// Input: `Operation => [Entity, ..]`. It implements
/// `OperationDeletes<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_deletes {
    ($Operation:ty => [$($Entity:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationDeletes<$Operation> for $Entity {}
        )+
    };
}

/// Declares, at compile time, that an operation may link the listed relations.
///
/// Input: `Operation => [Relation, ..]`. It implements
/// `OperationLinks<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_links {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationLinks<$Operation> for $Relation {}
        )+
    };
}

/// Declares, at compile time, that an operation may unlink the listed
/// relations.
///
/// Input: `Operation => [Relation, ..]`. It implements
/// `OperationUnlinks<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_unlinks {
    ($Operation:ty => [$($Relation:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationUnlinks<$Operation> for $Relation {}
        )+
    };
}

/// Declares, at compile time, that an operation may emit the listed external
/// effects.
///
/// Input: `Operation => [Effect, ..]`. It implements
/// `OperationEmits<Operation>` for each listed marker.
#[macro_export]
macro_rules! worth_query_operation_emits {
    ($Operation:ty => [$($Effect:ty),+ $(,)?]) => {
        $(
            impl $crate::facade::application_schema::OperationEmits<$Operation> for $Effect {}
        )+
    };
}

/// Dispatches one inline `worth_query_operation!` capability clause to its
/// standalone macro. An unknown clause is a compile error:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::*;
/// pub struct Board;
/// pub struct Input;
/// pub struct Task;
/// worth_query_declaration::worth_query_structured_value_binding!(pub InputBinding for Input {
///     identity: "inline-capability.input.v1"
/// });
/// worth_query_declaration::worth_query_operation!(pub Add for Board, input InputBinding;
///     reads [Task]; destroys [Task]
/// );
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_operation_inline_capability {
    (reads $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_reads!($Operation => [$($Member),*]);
    };
    (writes $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_writes!($Operation => [$($Member),*]);
    };
    (creates $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_creates!($Operation => [$($Member),*]);
    };
    (deletes $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_deletes!($Operation => [$($Member),*]);
    };
    (links $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_links!($Operation => [$($Member),*]);
    };
    (unlinks $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_unlinks!($Operation => [$($Member),*]);
    };
    (emits $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_emits!($Operation => [$($Member),*]);
    };
    (expects_version $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_expects_version!($Operation => [$($Member),*]);
    };
    (expects_fact $Operation:ident [$($Member:ty),*]) => {
        $crate::worth_query_operation_expects_fact!($Operation => [$($Member),*]);
    };
    ($kind:ident $Operation:ident [$($Member:ty),*]) => {
        ::core::compile_error!(concat!(
            "unknown operation capability clause `", stringify!($kind),
            "`; expected reads, writes, creates, deletes, links, unlinks, emits, ",
            "expects_version or expects_fact"
        ));
    };
}
