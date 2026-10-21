
Selection is indexed by the installed mutation binding, with its exact input
and scope types. It supplies no native authority.

```rust
use worth_query_declaration::facade::{application_operation::*, application_schema::*};
fn selected<S, B, I>(input: &I) -> B::ScopeBinding
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S, Input = I>,
    I: ApplicationMutationInputScope<S, B>,
{
    <I as ApplicationMutationInputScope<S, B>>::scope_for_binding(input)
}
```

An association for one binding cannot select another, even with identical input
and scope types:

```compile_fail,E0277
use worth_query_declaration::facade::{application_operation::*, application_schema::*};
fn wrong_binding<S, A, B, I>(input: &I)
where
    S: ApplicationSchema,
    A: ApplicationMutationBinding<S, Input = I>,
    B: ApplicationMutationBinding<S, Input = I>,
    I: ApplicationMutationInputScope<S, A>,
{
    let _ = <I as ApplicationMutationInputScope<S, B>>::scope_for_binding(input);
}
```

The binding's input type cannot be replaced by an unrelated input:

```compile_fail,E0271
use worth_query_declaration::facade::{application_operation::*, application_schema::*};
fn wrong_input<S, B>()
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S, Input = ()>,
    String: ApplicationMutationInputScope<S, B>,
{}
```

Its scope selection must have the declared scope type:

```compile_fail,E0308
use worth_query_declaration::facade::{application_operation::*, application_schema::*};
fn wrong_scope<S, B, I>(_: &I) -> B::ScopeBinding
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S, Input = I>,
    I: ApplicationMutationInputScope<S, B>,
{
    7_u64
}
```
