//! Every configured compiled marker binds the exact bytes compiled by this fixture.

#[cfg(not(target_family = "wasm"))]
#[test]
fn compiled_examples_match_consumer_documents() {
    use std::{collections::BTreeSet, fs, path::Path};
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .ancestors()
        .nth(7)
        .expect("fixture is inside the repository");
    let configuration: toml::Value = toml::from_str(
        &fs::read_to_string(root.join("tools/boundary-check/config/consumer-docs.toml"))
            .expect("consumer configuration is readable"),
    )
    .expect("consumer configuration is valid TOML");
    let mut examples = BTreeSet::new();
    for document in configuration["documents"]
        .as_array()
        .expect("exact document inventory")
    {
        let document = document.as_str().expect("document path");
        let text = normalized(
            &fs::read_to_string(root.join(document)).expect("consumer document is readable"),
        );
        for (name, block) in compiled_blocks(&text) {
            assert!(
                examples.insert(name.clone()),
                "duplicate compiled marker: {name}"
            );
            let source = normalized(
                &fs::read_to_string(manifest.join("src").join(&name))
                    .unwrap_or_else(|error| panic!("{name}: source is unreadable: {error}")),
            );
            compare_region(document, &name, &block, &source);
        }
    }
    assert_eq!(
        examples,
        BTreeSet::from([
            "warehouse_inventory.rs".to_owned(),
            "owning_entry.rs".to_owned(),
            "sensor_telemetry.rs".to_owned(),
            "checkpoint_recovery/computation_partition/demand.rs".to_owned(),
            "checkpoint_recovery/required_chain/concurrent_waves/application.rs".to_owned(),
        ]),
        "all configured compiled examples must retain their pinned source bindings"
    );
}

#[cfg(not(target_family = "wasm"))]
fn compiled_blocks(text: &str) -> Vec<(String, String)> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
    let mut blocks = Vec::new();
    let mut name = None;
    let mut block = String::new();
    for event in Parser::new(text) {
        match event {
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Some(value) = html
                    .trim()
                    .strip_prefix("<!-- compiled: ")
                    .and_then(|value| value.strip_suffix(" -->"))
                {
                    name = Some(value.to_owned());
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if name.is_some() => {
                assert_eq!(
                    info.as_ref(),
                    "rust",
                    "compiled example must be a plain Rust fence"
                );
            }
            Event::Text(text) if name.is_some() => block.push_str(&text),
            Event::End(TagEnd::CodeBlock) if name.is_some() => {
                blocks.push((name.take().unwrap(), std::mem::take(&mut block)));
            }
            _ => {}
        }
    }
    assert!(name.is_none(), "compiled marker has no code block");
    blocks
}

#[cfg(not(target_family = "wasm"))]
fn compare_region(document: &str, name: &str, block: &str, source: &str) {
    let compiled = source
        .split_once("// compiled-example: begin\n")
        .and_then(|(_, rest)| rest.split_once("// compiled-example: end\n"))
        .map(|(example, _)| example.trim_end_matches([' ', '\t']))
        .unwrap_or_else(|| panic!("{name}: source markers are missing"));
    if block != compiled {
        let doc_lines: Vec<_> = block.split_inclusive('\n').collect();
        let source_lines: Vec<_> = compiled.split_inclusive('\n').collect();
        let differing = (0..doc_lines.len().max(source_lines.len()))
            .find(|&line| doc_lines.get(line) != source_lines.get(line))
            .unwrap();
        panic!(
            "{document}: {name}: first differing line {}\ndocument: {:?}\nsource: {:?}",
            differing + 1,
            doc_lines.get(differing),
            source_lines.get(differing)
        );
    }
}

#[cfg(not(target_family = "wasm"))]
fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}
