//! Complete feature members lower through the existing declaration and Host owners.
mod members;

/// Declares one complete static feature definition.
///
/// The header names the definition's generics. The first parameter is the
/// schema: `Definition<S: Bound>` is generic over every schema satisfying
/// `Bound`; `Definition<MySchema>` targets one concrete schema. Further type
/// parameters (`O: Operation`) and a trailing `where` clause are carried onto
/// the definition, its runtime and every generated impl, so one declaration
/// can describe a family such as
/// `pub feature OperationFamily<S: FamilySchema, O: Operation> for OperationFeature<O> => OperationRuntime where O: Send { .. }`.
///
/// Runtime constructor arguments are explicit values for caller-supplied
/// handlers. A `body_action(Binding, Delivery)` declares its operation and
/// binding, derives its delivery root and installs the schema-owned
/// operation body through the contribution's existing body owner; it takes
/// no handler value.
/// A complete definition cannot substitute an operation-only roster for typed members.
/// ```compile_fail
/// use worth_query_execution::facade::declaration::{application_program::*, application_schema::*};
/// trait Bound: ApplicationSchema {}
/// struct Feature;
/// impl<S: Bound> ApplicationFeature<S> for Feature {
///     type Inputs = ApplicationFeatureInputLeaf;
///     const IDENTITY: &'static str = "complete.feature";
/// }
/// worth_query_execution::facade::worth_query_application_contribution! {
///     pub feature Wrong<S: Bound> for Feature => Runtime {
///         schema: |schema| { schema };
///         conditional_operation(());
///     }
/// }
/// ```
#[macro_export]
macro_rules! worth_query_application_contribution {
    ($vis:vis feature $Definition:ident<
        $Schema:ident: $Bound:path $(, $Param:ident $(: $ParamBound:path)?)* $(,)?
    > for $Feature:ty => $Runtime:ident { $($block:tt)* }) => {
        $crate::__worth_query_complete_feature!(@block
            [$vis $Definition [$Schema, $($Param,)*] [$Schema: $Bound, $($Param $(: $ParamBound)?,)*] []
                $Schema, $Feature, $Runtime] $($block)*
        );
    };
    ($vis:vis feature $Definition:ident<
        $Schema:ident: $Bound:path $(, $Param:ident $(: $ParamBound:path)?)* $(,)?
    > for $Feature:ty => $Runtime:ident where $($rest:tt)+) => {
        $crate::__worth_query_complete_feature!(@where
            [$vis $Definition [$Schema, $($Param,)*] [$Schema: $Bound, $($Param $(: $ParamBound)?,)*]]
            [$Schema, $Feature, $Runtime] [] $($rest)+
        );
    };
    ($vis:vis feature $Definition:ident<
        $Schema:ty $(, $Param:ident $(: $ParamBound:path)?)* $(,)?
    > for $Feature:ty => $Runtime:ident { $($block:tt)* }) => {
        $crate::__worth_query_complete_feature!(@block
            [$vis $Definition [$($Param,)*] [$($Param $(: $ParamBound)?,)*] []
                $Schema, $Feature, $Runtime] $($block)*
        );
    };
    ($vis:vis feature $Definition:ident<
        $Schema:ty $(, $Param:ident $(: $ParamBound:path)?)* $(,)?
    > for $Feature:ty => $Runtime:ident where $($rest:tt)+) => {
        $crate::__worth_query_complete_feature!(@where
            [$vis $Definition [$($Param,)*] [$($Param $(: $ParamBound)?,)*]]
            [$Schema, $Feature, $Runtime] [] $($rest)+
        );
    };
}
