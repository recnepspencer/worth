use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};

pub use worth_query_execution_lineage::facade::primary_graph::conditional_operation::operation_invocation::{
    WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind,
};

/// Application behavior invoked only after Query has freshly admitted the
/// exact installed operation. The invoker describes decision reads and domain
/// effects; Query owns temporal-intent advancement and the final commit.
pub trait WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>:
    Send + Sync + 'static
{
    const SEMANTIC_IDENTITY: &'static str;
    type Projection: 'static;

    fn preconditions(
        &self,
        _input: &Input,
    ) -> worth_query_declaration::facade::application_schema::TypedMutationPreconditions<
        Schema,
        Operation,
        Scope,
    > {
        Default::default()
    }

    fn project(
        &self,
        input: &Input,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        scope: &WorthQueryInvariantEntityIdentity<Schema, Scope>,
    ) -> Result<Self::Projection, WorthQueryTemporalInvocationFailure>;

    fn apply(
        &self,
        input: Input,
        projection: Self::Projection,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryTemporalInvocationFailure>;
}
