use super::*;

pub(super) fn task_insert(title: &str) -> crate::runtime::WorthQueryWriteCommand {
    insert_command(
        "Task",
        [
            ("identity.id", test_string_aspect_value("")),
            ("title.value", test_string_aspect_value(title)),
        ],
    )
}
