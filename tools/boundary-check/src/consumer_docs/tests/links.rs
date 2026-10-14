use super::*;

#[test]
fn missing_relative_file_and_heading_fail_bc8009() {
    let fixture = Fixture::new("# Guide\n[missing](absent.md)\n[wrong](#not-guide)\n", 0);
    assert_eq!(
        fixture.codes(),
        [
            DiagnosticCode::Bc8009ConsumerDocLink,
            DiagnosticCode::Bc8009ConsumerDocLink
        ]
    );
}

#[test]
fn relative_reference_links_images_and_github_duplicate_slugs_are_valid() {
    let fixture = Fixture::new("# Guide\n[heading](target.md#hello-world)\n[duplicate](target.md#hello-world-2)\n[reference][r]\n![image](pixel.png)\n[external](https://example.com/#absent)\n\n[r]: target.md#hello-world-1\n\n```text\n[ignored](absent.md)\n```\n", 0);
    fixture.write(
        "docs/target.md",
        "# Hello *World!*\n# Hello World-1\n# Hello `World!`\n",
    );
    fixture.write("docs/pixel.png", "image existence is the subject");
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn percent_encoded_paths_and_unicode_headings_resolve() {
    let fixture = Fixture::new("[unicode](a%20file.md#caf%C3%A9)\n", 0);
    fixture.write("docs/a file.md", "# Café\n");
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn github_preserves_marks_and_connectors_but_removes_separators_and_other_numbers() {
    let fixture = Fixture::new("[mark](target.md#cafe%CC%81)\n[connector](target.md#a%E2%80%BFb)\n[space](target.md#ab)\n[number](target.md#fraction)\n", 0);
    fixture.write(
        "docs/target.md",
        "# Cafe\u{301}\n# A‿B\n# A&nbsp;B\n# ⅓fraction\n",
    );
    assert!(fixture.diagnostics().is_empty());
    fixture.write(
        "docs/guide.md",
        "[wrong-space](target.md#a-b)\n[wrong-mark](target.md#cafe)\n",
    );
    assert_eq!(
        fixture.codes(),
        [
            DiagnosticCode::Bc8009ConsumerDocLink,
            DiagnosticCode::Bc8009ConsumerDocLink
        ]
    );
}
