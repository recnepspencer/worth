use std::fmt::Write;

pub(super) fn framed_parts(parts: &[&str]) -> String {
    let mut encoded = String::new();
    for part in parts {
        write!(&mut encoded, "{}:{part}", part.len())
            .expect("writing framed application-program text into a String cannot fail");
    }
    encoded
}

pub(super) fn framed_fields(fields: impl IntoIterator<Item = (&'static str, String)>) -> String {
    let mut encoded = String::new();
    for (index, (field, value)) in fields.into_iter().enumerate() {
        if index > 0 {
            encoded.push('|');
        }
        write!(&mut encoded, "{field}={}:{}", value.len(), value)
            .expect("writing framed application-program text into a String cannot fail");
    }
    encoded
}

pub(super) fn framed_record(
    family: &'static str,
    fields: impl IntoIterator<Item = (&'static str, String)>,
) -> String {
    format!("{family}|{}", framed_fields(fields))
}
