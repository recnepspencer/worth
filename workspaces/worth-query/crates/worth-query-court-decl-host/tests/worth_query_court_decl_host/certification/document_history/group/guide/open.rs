//! Normal group construction retains the original document on refusal.
use super::super::*;

pub(crate) fn open_group<'runtime>(
    document: Document<'runtime>,
    host: &App,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        DocumentRetentionSchema,
    >,
    key: u128,
    second: u64,
) -> Result<Group<'runtime>, (Document<'runtime>, WorthQueryHistoryGroupDenial)> {
    // compiled-example: begin
    document.group_edits::<_, RetentionProgramP0, Action, 2>(
        host,
        request,
        key,
        [
            (
                SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 3,
                    },
                },
                0x9182_5001,
            ),
            (
                SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: second,
                    },
                },
                0x9182_5002,
            ),
        ],
    )
    // compiled-example: end
}
