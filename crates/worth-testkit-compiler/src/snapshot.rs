//! Only unasserted external source coordinates use the historical snapshot mask.
use std::path::Path;

use crate::{
    diagnostic_layout,
    source_locations::{Location, SourceContext},
};

pub(crate) fn compared(
    expected: &str,
    observed: &str,
    source: &Path,
) -> Result<(String, String), String> {
    let context = SourceContext::new(source);
    let expected = crate::source_locations::normalize(expected, source);
    // First remove unasserted line numbers without changing the compiler's
    // gutter width. That retains the evidence needed by common-margin removal.
    let observed = mask_external(&context, &expected, observed, GutterWidth::Compiler)?;
    let expected = diagnostic_layout::unindent(&expected);
    let observed = diagnostic_layout::unindent(&observed);
    // Only now align masked external gutters with the normalized snapshot.
    // Doing so earlier makes them influence the primary source's common margin.
    let observed = mask_external(&context, &expected, &observed, GutterWidth::Snapshot)?;
    let observed = crate::diagnostic_lists::normalize(&expected, &observed);
    Ok((expected, observed))
}

enum GutterWidth {
    Compiler,
    Snapshot,
}

fn mask_external(
    context: &SourceContext,
    expected: &str,
    observed: &str,
    width: GutterWidth,
) -> Result<String, String> {
    let lines = expected.lines().collect::<Vec<_>>();
    let headers = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            context.location(line).map(|location| {
                let gutter = lines.get(index + 1).and_then(|line| snippet_width(line));
                (location, gutter)
            })
        })
        .collect::<Vec<_>>();
    let mut index = 0;
    let mut hide_gutter = None;
    let mut actual = Vec::new();
    for line in observed.replace("\r\n", "\n").lines() {
        if let Some(mut location) = context.location(line) {
            if !location.known {
                return Err(format!(
                    "unrecognized compiler source location {:?}",
                    location.path
                ));
            }
            hide_gutter = None;
            if let Some((expected, gutter)) = headers.get(index) {
                location.path = registry_version(&location.path, &expected.path);
                if location.external
                    && location.path == expected.path
                    && expected.coordinates.is_empty()
                {
                    location.coordinates.clear();
                    hide_gutter = *gutter;
                }
            }
            index += 1;
            actual.push(render(&location));
        } else if line.trim_start().starts_with("...") {
            // Elision markers do not end the current external snippet. Their
            // alignment belongs to the compiler, not its line-number gutter.
            actual.push(line.to_owned());
        } else if let Some((expected_width, actual_width)) = hide_gutter.zip(snippet_width(line)) {
            let preserved_width = match width {
                GutterWidth::Compiler => actual_width,
                GutterWidth::Snapshot => expected_width,
            };
            actual.push(format!(
                "{}{}",
                " ".repeat(preserved_width),
                &line[actual_width..]
            ));
        } else {
            hide_gutter = None;
            actual.push(line.to_owned());
        }
    }
    Ok(actual.join("\n"))
}

fn render(location: &Location) -> String {
    format!(
        "{}{}{}",
        location.prefix, location.path, location.coordinates
    )
}

fn snippet_width(line: &str) -> Option<usize> {
    let width = line
        .bytes()
        .take_while(|byte| *byte == b' ' || byte.is_ascii_digit())
        .count();
    let tail = &line[width..];
    (tail == "|" || tail.starts_with("| ")).then_some(width)
}

fn registry_version(observed: &str, expected: &str) -> String {
    if let Some((package, relative)) = expected
        .strip_prefix("$CARGO/")
        .and_then(|value| value.split_once("-$VERSION/"))
    {
        let prefix = format!("$CARGO/{package}-");
        if let Some((version, path)) = observed
            .strip_prefix(&prefix)
            .and_then(|value| value.split_once('/'))
        {
            if path == relative
                && version.starts_with(|value: char| value.is_ascii_digit())
                && version.bytes().all(|value| {
                    value.is_ascii_alphanumeric() || matches!(value, b'.' | b'-' | b'+')
                })
            {
                return expected.to_owned();
            }
        }
    }
    observed.to_owned()
}
