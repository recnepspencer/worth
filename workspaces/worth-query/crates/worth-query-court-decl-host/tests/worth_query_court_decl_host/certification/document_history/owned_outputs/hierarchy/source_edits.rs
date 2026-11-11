//! Real ordinary source edits establish stale publications before history attaches.
use super::*;
macro_rules! source_edit {
    ($host:ident, $request:ident, $input:expr, $action:ty, $key:ident) => {{
        let mut operation = $request
            .mutate($input)
            .without_source()
            .idempotency_owned($key.into())
            .unwrap_or_else(|(_, _, cause)| panic!("{cause:?}"))
            .retain_operation::<P, $action>($host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        for count in 0..128 {
            assert!(count < 127, "ordinary source edit must complete");
            operation = match operation.advance($request) {
                WorthQuerySourceOnlyAdvance::Progress(next) => next,
                WorthQuerySourceOnlyAdvance::Completed(done) => {
                    done.close($request)
                        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                    return;
                }
                WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("{denial:?}"),
                WorthQuerySourceOnlyAdvance::Terminal(_) => panic!("source edit terminated"),
            };
        }
    }};
}
pub(super) fn model<P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    request: &family::Request<'_, '_, '_>,
    value: u64,
    key: &str,
) {
    source_edit!(
        host,
        request,
        edit::Intent {
            key: "a".into(),
            value
        },
        super::super::program::EditAction,
        key
    );
}
pub(super) fn member<P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    request: &family::Request<'_, '_, '_>,
    value: u64,
    key: &str,
) {
    source_edit!(
        host,
        request,
        member_edit::Intent {
            key: "m".into(),
            value
        },
        super::super::program::MemberEditAction,
        key
    );
}
