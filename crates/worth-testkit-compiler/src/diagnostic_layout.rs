//! Remove only the renderer's common left margin, never spacing after a gutter.
//! This is the stable unindent rule used by the prior snapshot engine.
enum LineKind {
    Heading,
    Code(usize),
    Note,
    Other(usize),
}

pub(crate) fn unindent(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        result.push(line.to_owned());
        index += 1;
        if !matches!(kind(line, true, &mut false), LineKind::Heading) {
            continue;
        }
        let Some(next) = lines.get(index) else {
            continue;
        };
        let LineKind::Code(mut margin) = kind(next, false, &mut false) else {
            continue;
        };
        if !next[margin + 1..].starts_with("--> ") {
            continue;
        }
        let mut end = index + 1;
        let mut previous_note = false;
        while let Some(line) = lines.get(end) {
            match kind(line, false, &mut previous_note) {
                LineKind::Heading => break,
                LineKind::Code(width) => margin = margin.min(width),
                LineKind::Note => (),
                LineKind::Other(width) => {
                    if width <= 10 {
                        break;
                    }
                }
            }
            end += 1;
        }
        previous_note = false;
        for line in &lines[index..end] {
            match kind(line, false, &mut previous_note) {
                LineKind::Code(_) | LineKind::Other(_) => {
                    if let Some(space) = line.find(' ') {
                        result.push(format!("{}{}", &line[..space], &line[space + margin..]));
                    } else {
                        result.push((*line).to_owned());
                    }
                }
                LineKind::Heading | LineKind::Note => result.push((*line).to_owned()),
            }
        }
        index = end;
    }
    result.join("\n").trim_end_matches('\n').to_owned()
}

fn kind(line: &str, first: bool, previous_note: &mut bool) -> LineKind {
    let was_note = std::mem::replace(previous_note, false);
    if ["error", "warning"].iter().any(|heading| {
        line.strip_prefix(heading)
            .is_some_and(|tail| tail.starts_with(':') || tail.starts_with('['))
    }) || first && line.starts_with("note: ")
    {
        return LineKind::Heading;
    }
    if line.starts_with("note:")
        || line == "..."
        || line.starts_with("help:")
        || was_note && line.starts_with("      ")
    {
        *previous_note = true;
        return LineKind::Note;
    }
    if let Some(tail) = line.strip_prefix("... ") {
        return LineKind::Code(tail.bytes().take_while(|byte| *byte == b' ').count());
    }
    let initial = line.bytes().take_while(|byte| *byte == b' ').count();
    let digits = line[initial..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    let spaces = initial
        + line[initial + digits..]
            .bytes()
            .take_while(|byte| *byte == b' ')
            .count();
    let tail = &line[digits + spaces..];
    if spaces > 0
        && (tail == "|"
            || tail.starts_with("| ")
            || digits > 0
                && ["~", "+", "-"]
                    .iter()
                    .any(|marker| tail == *marker || tail.starts_with(&format!("{marker} ")))
            || digits == 0
                && ["--> ", "::: ", "= "]
                    .iter()
                    .any(|marker| tail.starts_with(marker)))
    {
        return LineKind::Code(spaces - 1);
    }
    LineKind::Other(if digits == 0 { spaces } else { 0 })
}
