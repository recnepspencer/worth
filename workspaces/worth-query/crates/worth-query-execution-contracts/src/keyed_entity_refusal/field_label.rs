//! A keyed refusal's field as words an end user reads, naming its entity once.
//!
//! Declared names are PascalCase or snake_case and often repeat their entity
//! (`OrderOrderId` on `Order`), so naive splitting reads "order order id".

/// `entity`'s words, then `field`'s words without any leading repetitions of
/// the entity's words.
pub(super) fn qualified_field_label(entity: &str, field: &str) -> String {
    let entity = words(entity);
    let mut field = words(field);
    while !entity.is_empty() && field.starts_with(&entity) {
        field.drain(..entity.len());
    }
    entity
        .into_iter()
        .chain(field)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lowercase words of a declared name. `_`, `-` and spaces separate words; a
/// capital starts one, except inside an acronym (`SKUCode` is `sku code`);
/// digits stay with the word before them (`Line2Total` is `line2 total`).
fn words(name: &str) -> Vec<String> {
    let characters: Vec<char> = name.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for (index, &character) in characters.iter().enumerate() {
        if matches!(character, '_' | '-' | ' ') {
            flush(&mut current, &mut words);
            continue;
        }
        if character.is_uppercase() && starts_word(&characters, index) {
            flush(&mut current, &mut words);
        }
        current.extend(character.to_lowercase());
    }
    flush(&mut current, &mut words);
    words
}

/// A capital starts a word after a lowercase letter or digit, or when it is
/// the last capital of an acronym followed by a lowercase letter.
fn starts_word(characters: &[char], index: usize) -> bool {
    let Some(&previous) = index
        .checked_sub(1)
        .and_then(|before| characters.get(before))
    else {
        return false;
    };
    let next_is_lower = characters
        .get(index + 1)
        .is_some_and(|next| next.is_lowercase());
    previous.is_lowercase()
        || previous.is_ascii_digit()
        || (previous.is_uppercase() && next_is_lower)
}

fn flush(current: &mut String, words: &mut Vec<String>) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}
