use std::marker::PhantomData;

use super::WorthQueryConditionalNodeLocation;

type ConditionalNodeMarker<D, O, F, N> = fn() -> (D, O, F, N);

/// Typed declaration reference for one operation or workflow-stage
/// conditional node.
///
/// The retained location is portable meaning, not runtime authority. Exact
/// operation, package, runtime, and generation affinity are established only
/// when the installed package index resolves this reference.
pub struct WorthQueryConditionalNodeRef<D, O, F, N> {
    location: WorthQueryConditionalNodeLocation,
    marker: PhantomData<ConditionalNodeMarker<D, O, F, N>>,
}

impl<D, O, F, N> WorthQueryConditionalNodeRef<D, O, F, N> {
    #[doc(hidden)]
    pub fn from_declared_location(location: WorthQueryConditionalNodeLocation) -> Self {
        Self {
            location,
            marker: PhantomData,
        }
    }

    pub fn location(&self) -> &WorthQueryConditionalNodeLocation {
        &self.location
    }

    pub fn node_identity(&self) -> &str {
        self.location.node_identity()
    }

    pub fn stage_identity(&self) -> Option<&str> {
        self.location.stage_identity()
    }
}

impl<D, O, F, N> Clone for WorthQueryConditionalNodeRef<D, O, F, N> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            marker: PhantomData,
        }
    }
}

impl<D, O, F, N> std::fmt::Debug for WorthQueryConditionalNodeRef<D, O, F, N> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryConditionalNodeRef")
            .field("location", &self.location)
            .finish_non_exhaustive()
    }
}

impl<D, O, F, N> PartialEq for WorthQueryConditionalNodeRef<D, O, F, N> {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl<D, O, F, N> Eq for WorthQueryConditionalNodeRef<D, O, F, N> {}

/// Declares a typed marker for one conditional node of a graph-read domain
/// operation, located either in the operation itself or in one workflow stage.
///
/// Input: `vis Node in Domain, Operation, Family => operation "node"`, or
/// `=> workflow_stage "stage", "node"`. It generates a unit struct `Node` with
/// a `fn reference()` returning its typed `WorthQueryConditionalNodeRef`; that
/// function panics if an identity literal is not a valid node or stage
/// identity. The reference is portable meaning; the installed package index
/// resolves it.
#[macro_export]
macro_rules! worth_query_conditional_node {
    (
        $vis:vis $Node:ident in $Domain:ty, $Operation:ty, $Family:ty
        => operation $identity:literal
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Node;

        impl $Node {
            pub fn reference() -> $crate::facade::WorthQueryConditionalNodeRef<
                $Domain,
                $Operation,
                $Family,
                Self,
            > {
                $crate::facade::WorthQueryConditionalNodeRef::from_declared_location(
                    $crate::facade::WorthQueryConditionalNodeLocation::operation($identity)
                        .expect("a declared operation conditional-node identity is valid"),
                )
            }
        }
    };
    (
        $vis:vis $Node:ident in $Domain:ty, $Operation:ty, $Family:ty
        => workflow_stage $stage:literal, $identity:literal
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Node;

        impl $Node {
            pub fn reference() -> $crate::facade::WorthQueryConditionalNodeRef<
                $Domain,
                $Operation,
                $Family,
                Self,
            > {
                $crate::facade::WorthQueryConditionalNodeRef::from_declared_location(
                    $crate::facade::WorthQueryConditionalNodeLocation::workflow_stage(
                        $stage,
                        $identity,
                    )
                    .expect("declared workflow-stage conditional-node identities are valid"),
                )
            }
        }
    };
}
