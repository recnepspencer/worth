/// Authors workflow meaning through the same public builder used without the macro.
///
/// The block form deliberately exposes the ordinary builder instead of maintaining
/// a second macro-only grammar or validator.
#[macro_export]
macro_rules! worth_query_workflow {
    (
        spec: $spec:ty;
        identity: $identity:expr;
        limits: $limits:expr;
        build: |$builder:ident| $body:block
    ) => {{
        let mut $builder = $crate::facade::application_program::ApplicationWorkflowDefinitionBuilder::<$spec>::new(
            $identity,
            $limits,
        )?;
        $body
        $builder.finish()
    }};
}
