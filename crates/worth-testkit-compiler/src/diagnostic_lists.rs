//! Preserve the eight named implementors asserted before an existing `$N` tail.
struct ListRule {
    heading: String,
    masked_tail: bool,
}

pub(crate) fn normalize(expected: &str, observed: &str) -> String {
    let expected_lines = expected.lines().collect::<Vec<_>>();
    let mut rules = Vec::new();
    for (index, line) in expected_lines.iter().enumerate() {
        if heading(line) {
            let tail = expected_lines.get(index + 9).map(|line| line.trim_start());
            rules.push(ListRule {
                heading: (*line).to_owned(),
                masked_tail: tail == Some("and $N others"),
            });
        }
    }
    let lines = observed.lines().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut cursor = 0;
    let mut list_index = 0;
    while cursor < lines.len() {
        let line = lines[cursor];
        result.push(line.to_owned());
        cursor += 1;
        if !heading(line) {
            continue;
        }
        let rule = rules.get(list_index);
        list_index += 1;
        if !rule.is_some_and(|rule| rule.masked_tail && rule.heading == line) {
            continue;
        }
        let start = cursor;
        while let Some(entry) = lines.get(cursor) {
            if entry.bytes().take_while(|byte| *byte == b' ').count() < 12 {
                break;
            }
            cursor += 1;
        }
        let count = cursor - start;
        if count > 8 {
            result.extend(
                lines[start..start + 8]
                    .iter()
                    .map(|line| (*line).to_owned()),
            );
            let indent = lines[start + 8]
                .bytes()
                .take_while(|byte| *byte == b' ')
                .count();
            result.push(format!("{}and $N others", " ".repeat(indent - 2)));
        } else {
            result.extend(lines[start..cursor].iter().map(|line| (*line).to_owned()));
            if count == 8 {
                if let Some(line) = lines.get(cursor) {
                    let tail = line.trim_start();
                    if tail
                        .strip_prefix("and ")
                        .and_then(|tail| tail.strip_suffix(" others"))
                        .is_some_and(|number| {
                            !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
                        })
                    {
                        let indent = line.len() - tail.len();
                        result.push(format!("{}and $N others", " ".repeat(indent)));
                        cursor += 1;
                    }
                }
            }
        }
    }
    masked_counts(expected, &result.join("\n"))
}

fn heading(line: &str) -> bool {
    let tail = line.trim_start();
    tail.starts_with("= help: the following types implement trait ")
        || tail.starts_with("= help: the following other types implement trait ")
}

fn masked_counts(expected: &str, observed: &str) -> String {
    let rules = expected
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("and ")
                .and_then(|tail| tail.strip_suffix(" others"))
        })
        .collect::<Vec<_>>();
    let mut index = 0;
    observed
        .lines()
        .map(|line| {
            let tail = line.trim_start();
            let count = tail
                .strip_prefix("and ")
                .and_then(|tail| tail.strip_suffix(" others"));
            let Some(count) = count else {
                return line.to_owned();
            };
            let rule = rules.get(index);
            index += 1;
            if rule == Some(&"$N")
                && !count.is_empty()
                && count.bytes().all(|byte| byte.is_ascii_digit())
            {
                let indent = line.len() - tail.len();
                format!("{}and $N others", " ".repeat(indent))
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
