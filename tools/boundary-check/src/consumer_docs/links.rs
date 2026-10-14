//! Relative Markdown destinations and GitHub heading anchors.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Targets {
    headings: BTreeMap<PathBuf, BTreeSet<String>>,
}

impl Targets {
    pub(super) fn validate(&mut self, root: &Path, document: &str, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (event, range) in Parser::new_ext(text, Options::ENABLE_TABLES).into_offset_iter() {
            let destination = match event {
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => dest_url,
                _ => continue,
            };
            if external(&destination) {
                continue;
            }
            if let Err(error) = self.check(root, document, &destination) {
                let line = text[..range.start]
                    .bytes()
                    .filter(|&byte| byte == b'\n')
                    .count()
                    + 1;
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc8009ConsumerDocLink,
                    format!("{document}:{line}"),
                    format!("{destination}: {error}"),
                ));
            }
        }
        diagnostics
    }

    fn check(&mut self, root: &Path, document: &str, destination: &str) -> Result<(), String> {
        let (path, fragment) = destination
            .split_once('#')
            .map_or((destination, None), |(path, anchor)| (path, Some(anchor)));
        let path = decode(path.split('?').next().unwrap_or(path))?;
        let target = if path.is_empty() {
            root.join(document)
        } else if path.starts_with('/') {
            root.join(path.trim_start_matches('/'))
        } else {
            root.join(document).parent().unwrap().join(path)
        };
        let target = target
            .canonicalize()
            .map_err(|_| "relative target does not exist".to_owned())?;
        if !target.starts_with(root.canonicalize().map_err(|error| error.to_string())?) {
            return Err("relative target escapes the repository".into());
        }
        let Some(fragment) = fragment.filter(|fragment| !fragment.is_empty()) else {
            return Ok(());
        };
        let anchor = decode(fragment)?;
        if !self.headings.contains_key(&target) {
            let target_text = crate::source_documents::read_to_string(&target)
                .map_err(|error| error.to_string())?;
            self.headings.insert(target.clone(), anchors(&target_text));
        }
        if self.headings[&target].contains(&anchor) {
            Ok(())
        } else {
            Err(format!("GitHub heading anchor #{anchor} does not exist"))
        }
    }
}

fn external(destination: &str) -> bool {
    destination.starts_with("//")
        || destination.split_once(':').is_some_and(|(scheme, _)| {
            !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
        })
}

fn decode(value: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let mut input = value.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = input.next().and_then(|byte| (byte as char).to_digit(16));
            let low = input.next().and_then(|byte| (byte as char).to_digit(16));
            let (Some(high), Some(low)) = (high, low) else {
                return Err("invalid percent encoding".into());
            };
            bytes.push((high * 16 + low) as u8);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

fn anchors(text: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut heading = None;
    for event in Parser::new_ext(text, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH) {
        match event {
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::Text(value) | Event::Code(value) if heading.is_some() => {
                heading.as_mut().unwrap().push_str(&value)
            }
            Event::SoftBreak | Event::HardBreak if heading.is_some() => {
                heading.as_mut().unwrap().push(' ')
            }
            Event::End(TagEnd::Heading(_)) => {
                let base: String = heading
                    .take()
                    .unwrap()
                    .to_lowercase()
                    .chars()
                    .filter(|character| retained_in_github_slug(*character))
                    .map(|character| if character == ' ' { '-' } else { character })
                    .collect();
                let mut slug = base.clone();
                let mut suffix = 0;
                while anchors.contains(&slug) {
                    suffix += 1;
                    slug = format!("{base}-{suffix}");
                }
                anchors.insert(slug);
            }
            _ => {}
        }
    }
    anchors
}

// GitHub slugger's category policy preserves marks and connector punctuation;
// only ordinary ASCII spaces become dashes. Alphabetic symbols are retained.
// https://github.com/Flet/github-slugger/blob/master/script/generate-regex.js
fn retained_in_github_slug(character: char) -> bool {
    use unicode_general_category::{get_general_category, GeneralCategory::*};
    character.is_alphabetic()
        || matches!(character, ' ' | '-')
        || matches!(
            get_general_category(character),
            UppercaseLetter
                | LowercaseLetter
                | TitlecaseLetter
                | ModifierLetter
                | OtherLetter
                | NonspacingMark
                | SpacingMark
                | EnclosingMark
                | DecimalNumber
                | LetterNumber
                | ConnectorPunctuation
        )
}
