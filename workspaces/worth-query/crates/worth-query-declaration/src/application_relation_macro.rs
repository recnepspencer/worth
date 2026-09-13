#[macro_export]
macro_rules! worth_query_relation {
    ($vis:vis $Relation:ident in $Schema:ty, $From:ty => $To:ty; integrity = same_context_no_self_edges_unbounded_retain_dangling) => {
        $crate::worth_query_relation! {
            @declare $vis $Relation in $Schema, $From => $To,
            $crate::facade::application_schema::ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
        }
    };
    ($vis:vis $Relation:ident in $Schema:ty, $From:ty => $To:ty; integrity = same_context_unbounded_retain_dangling) => {
        $crate::worth_query_relation! {
            @declare $vis $Relation in $Schema, $From => $To,
            $crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        }
    };
    ($vis:vis $Relation:ident in $Schema:ty, $From:ty => $To:ty; integrity = $integrity:expr) => {
        $crate::worth_query_relation!(@declare $vis $Relation in $Schema, $From => $To, $integrity);
    };
    (@declare $vis:vis $Relation:ident in $Schema:ty, $From:ty => $To:ty, $integrity:expr) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Relation;

        impl $Relation {
            pub const fn reference() -> $crate::facade::application_schema::ApplicationRelationRef<$Schema, Self, $From, $To> {
                $crate::facade::application_schema::ApplicationRelationRef::from_schema_identifiers(
                    stringify!($Relation),
                    stringify!($From),
                    stringify!($To),
                    $integrity,
                )
            }
        }
    };
}
