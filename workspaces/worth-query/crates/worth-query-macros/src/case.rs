//! Identifier case conversion for names a derive generates from Rust names.

/// `task_key` becomes `TaskKey`.
pub(crate) fn pascal_case(name: &str) -> String {
    name.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            characters
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + characters.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// `TaskRow` becomes `task_row`.
pub(crate) fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, character) in name.char_indices() {
        if character.is_ascii_uppercase() && index > 0 && !name[..index].ends_with('_') {
            snake.push('_');
        }
        snake.push(character.to_ascii_lowercase());
    }
    snake
}

/// `task_key` becomes `TASK_KEY`.
pub(crate) fn screaming_snake_case(name: &str) -> String {
    snake_case(name).to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_names_become_type_names_and_type_names_become_module_names() {
        assert_eq!(pascal_case("task_key"), "TaskKey");
        assert_eq!(pascal_case("due"), "Due");
        assert_eq!(pascal_case("value_2"), "Value2");
        assert_eq!(snake_case("TaskRow"), "task_row");
        assert_eq!(snake_case("Row"), "row");
        assert_eq!(snake_case("Task_Row"), "task_row");
        assert_eq!(screaming_snake_case("task_key"), "TASK_KEY");
        assert_eq!(screaming_snake_case("due"), "DUE");
    }
}
