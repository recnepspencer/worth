//! The guide's marked fences are the exact sources compiled by this fixture.

use std::{fs, path::Path};

#[test]
fn compiled_examples_match_build_guide() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .ancestors()
        .nth(7)
        .expect("fixture is inside the repository");
    let guide = normalized(
        &fs::read_to_string(root.join("docs/build-an-application.md"))
            .expect("the build guide is readable"),
    );
    let mut examples = Vec::new();
    let mut lines = guide.split_inclusive('\n');
    while let Some(line) = lines.next() {
        let Some(name) = line
            .strip_prefix("<!-- compiled: ")
            .and_then(|line| line.strip_suffix(" -->\n"))
        else {
            continue;
        };
        assert_eq!(
            lines.next(),
            Some("```rust\n"),
            "{name}: marker must precede a Rust fence"
        );
        let mut block = String::new();
        loop {
            let line = lines
                .next()
                .unwrap_or_else(|| panic!("{name}: unclosed fence"));
            if line == "```\n" {
                break;
            }
            block.push_str(line);
        }
        let source = normalized(
            &fs::read_to_string(manifest.join("src").join(name))
                .unwrap_or_else(|error| panic!("{name}: source is unreadable: {error}")),
        );
        let compiled = source
            .split_once("// compiled-example: begin\n")
            .and_then(|(_, rest)| rest.split_once("// compiled-example: end\n"))
            .map(|(example, _)| example)
            .unwrap_or_else(|| panic!("{name}: source markers are missing"));
        if block != compiled {
            let guide_lines: Vec<_> = block.split_inclusive('\n').collect();
            let source_lines: Vec<_> = compiled.split_inclusive('\n').collect();
            let differing = (0..guide_lines.len().max(source_lines.len()))
                .find(|&line| guide_lines.get(line) != source_lines.get(line))
                .unwrap();
            panic!(
                "{name}: first differing line {}\nguide: {:?}\nsource: {:?}",
                differing + 1,
                guide_lines.get(differing),
                source_lines.get(differing)
            );
        }
        examples.push(name);
    }
    assert_eq!(
        examples,
        ["warehouse_inventory.rs", "sensor_telemetry.rs"],
        "both examples must retain their compiled markers"
    );
}

fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}
