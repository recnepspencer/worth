//! Member accumulation for `worth_query_application_contribution!`.
//!
//! The header is `[vis Definition [names] [declared generics] [where] Schema,
//! Feature, Runtime, schema body]`. Members accumulate, in this order, into
//! actions (schema and output roots), handler fields, body bindings, required
//! pairs, bound queries, managed indexes, invariants, action-reference aliases
//! and the feature spec, then lower through
//! `__worth_query_complete_feature_lowering!` to the same public traits a
//! hand-written feature implements.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_complete_feature {
    (@where $generics:tt [$($tail:tt)*] [$($where:tt)*] { $($block:tt)* }) => {
        $crate::__worth_query_complete_feature!(@where_block $generics [$($where)*] [$($tail)*]
            $($block)*);
    };
    (@where $generics:tt $tail:tt [$($where:tt)*] $next:tt $($rest:tt)+) => {
        $crate::__worth_query_complete_feature!(@where $generics $tail [$($where)* $next] $($rest)+);
    };
    (@where_block [$vis:vis $Definition:ident $names:tt $declared:tt] $where:tt
        [$Schema:ty, $Feature:ty, $Runtime:ident] $($block:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@block
            [$vis $Definition $names $declared $where $Schema, $Feature, $Runtime] $($block)*);
    };
    (@block [$($head:tt)*] schema: |$schema:ident| $body:block; $($members:tt)*) => {
        $crate::__worth_query_complete_feature!(@members [$($head)*, $schema $body]
            [] [] [] [] [] [] [] [] []; $($members)*
        );
    };
    (@block [$($head:tt)*] $($members:tt)*) => {
        $crate::__worth_query_complete_feature!(@members
            [$($head)*, __worth_query_schema { __worth_query_schema }]
            [] [] [] [] [] [] [] [] []; $($members)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt $invariants:tt [$($aliases:tt)*] [$($spec:tt)*];
        action $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            operation: $operation:expr; $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare $operation])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes $invariants
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt $invariants:tt [$($aliases:tt)*] [$($spec:tt)*];
        action $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare_pure])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes $invariants
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] [$($handlers:tt)*] $bodies:tt $roles:tt $queries:tt
        $indexes:tt $invariants:tt [$($aliases:tt)*] [$($spec:tt)*];
        action_ref $field:ident($Binding:ty, $Delivery:ty, $Handler:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [owned])]
            [$($handlers)* ($field, $Binding, $Handler)] $bodies $roles $queries $indexes $invariants
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] $handlers:tt [$($bodies:tt)*] $roles:tt $queries:tt
        $indexes:tt $invariants:tt [$($aliases:tt)*] [$($spec:tt)*];
        body_action($Binding:ty, $Delivery:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            operation: $operation:expr; $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [declare $operation])]
            $handlers [$($bodies)* ($Binding)] $roles $queries $indexes $invariants
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt [$($actions:tt)*] $handlers:tt [$($bodies:tt)*] $roles:tt $queries:tt
        $indexes:tt $invariants:tt [$($aliases:tt)*] [$($spec:tt)*];
        body_action_ref($Binding:ty, $Delivery:ty) $(as $Alias:ident $(<$($AliasGen:ident),* $(,)?>)?)? {
            $($clauses:tt)*
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head
            [$($actions)* ($Binding, $Delivery, [owned])]
            $handlers [$($bodies)* ($Binding)] $roles $queries $indexes $invariants
            [$($aliases)* $(($Alias, [$($($AliasGen),*)?], $Binding, $Delivery))?]
            [$($spec)* action($Binding, $Delivery) { $($clauses)* };]; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt [$($queries:tt)*] $indexes:tt
        $invariants:tt $aliases:tt $spec:tt;
        query($QueryBinding:ty) { definition: $definition:expr; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            [$($queries)* ($QueryBinding, $definition)] $indexes $invariants $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $invariants:tt $aliases:tt $spec:tt;
        managed_indexes: [$($Index:ty),* $(,)?]; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* $(($Index, [auto]))*] $invariants $aliases $spec; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $invariants:tt $aliases:tt $spec:tt;
        managed_index($Index:ty) { provider: $provider:expr; watches: true; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* ($Index, [custom_watches $provider])] $invariants $aliases $spec;
            $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt [$($indexes:tt)*]
        $invariants:tt $aliases:tt $spec:tt;
        managed_index($Index:ty) { provider: $provider:expr; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries [$($indexes)* ($Index, [custom $provider])] $invariants $aliases $spec;
            $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt $indexes:tt
        [$($invariants:tt)*] $aliases:tt $spec:tt;
        invariant($Invariant:ty) { definition: $definition:expr; rule: $rule:expr; }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries $indexes [$($invariants)* ($Invariant, $definition, $rule)] $aliases $spec;
            $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt [$($roles:tt)*] $queries:tt $indexes:tt
        $invariants:tt $aliases:tt [$($spec:tt)*];
        required_pair $field:ident($Roles:ty, $InitialHandler:ty, $PreserveHandler:ty) {
            initial_operation: $initial:expr;
            preserve_operation: $preserve:expr;
        }; $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies
            [$($roles)* ($field, $Roles, $InitialHandler, $PreserveHandler, $initial, $preserve)]
            $queries $indexes $invariants $aliases
            [$($spec)* required_pair($Roles);]; $($rest)*
        );
    };
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt $indexes:tt
        $invariants:tt $aliases:tt [$($spec:tt)*];
        $kind:ident($Member:ty); $($rest:tt)*
    ) => {
        $crate::__worth_query_complete_feature!(@descriptive $kind);
        $crate::__worth_query_complete_feature!(@members $head $actions $handlers $bodies $roles
            $queries $indexes $invariants $aliases
            [$($spec)* $kind($Member);]; $($rest)*
        );
    };
    (@descriptive provides) => {};
    (@descriptive derived_artifact) => {};
    (@descriptive derived_collection) => {};
    (@descriptive managed_computation) => {};
    (@members $head:tt $actions:tt $handlers:tt $bodies:tt $roles:tt $queries:tt $indexes:tt
        $invariants:tt $aliases:tt $spec:tt;
    ) => {
        $crate::__worth_query_complete_feature_lowering!($head $actions $handlers $bodies $roles
            $queries $indexes $invariants $aliases $spec);
    };
}
