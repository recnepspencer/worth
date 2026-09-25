use std::path::PathBuf;

use crate::{
    UiDslComponentReference, UiLayoutCell, UiLayoutDeclaration, UiLayoutGrid, UiLayoutTrack,
    UiLayoutWidthInterval, WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode,
    WorthUiDslCompileReport, WorthUiDslCompiler, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule, WorthUiSealedSemanticPackage,
};

const SOURCE: &str = "
layout demo.component.row {
  columns flex 1 min 120;
  rows fixed 40, fixed 40;
  gap 0 8;
  padding 12 6;
  member demo.component.label at 0 0;
  member demo.component.icon at 0 1;
  width from 600 to 900 {
    columns flex 3 min 200 max 640, fixed 48;
    rows fixed 40;
    padding 12 6;
    member demo.component.label at 0 0;
    member demo.component.icon at 1 0;
  }
  width from 900 {
    columns flex 3, fixed 48;
    rows fixed 40;
    gap 16 0;
    member demo.component.label at 0 0 span 2 1;
    member demo.component.icon at 1 0;
  }
}
";

fn component(value: &str) -> UiDslComponentReference {
    UiDslComponentReference::new(value).expect("the fixture component reference is valid")
}

/// The declaration `SOURCE` spells, built through the Rust vocabulary.
fn declaration() -> UiLayoutDeclaration {
    let label = component("demo.component.label");
    let icon = component("demo.component.icon");
    let flex = |weight, min, max| UiLayoutTrack::Flexible { weight, min, max };
    let row = UiLayoutTrack::Fixed { extent: 40 };
    UiLayoutDeclaration::new(
        component("demo.component.row"),
        UiLayoutGrid::new([flex(1, 120, None)], [row, row])
            .with_gaps(0, 8)
            .with_padding(12, 6)
            .with_member(label.clone(), UiLayoutCell::at(0, 0))
            .with_member(icon.clone(), UiLayoutCell::at(0, 1)),
    )
    .with_variant(
        UiLayoutWidthInterval::between(600, 900),
        UiLayoutGrid::new(
            [flex(3, 200, Some(640)), UiLayoutTrack::Fixed { extent: 48 }],
            [row],
        )
        .with_padding(12, 6)
        .with_member(label.clone(), UiLayoutCell::at(0, 0))
        .with_member(icon.clone(), UiLayoutCell::at(1, 0)),
    )
    .with_variant(
        UiLayoutWidthInterval::at_least(900),
        UiLayoutGrid::new(
            [flex(3, 0, None), UiLayoutTrack::Fixed { extent: 48 }],
            [row],
        )
        .with_gaps(16, 0)
        .with_member(label, UiLayoutCell::spanning(0, 0, 2, 1))
        .with_member(icon, UiLayoutCell::at(1, 0)),
    )
}

fn compile(source: &str) -> Result<WorthUiSealedSemanticPackage, WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
}

fn sealed_layouts(package: &WorthUiSealedSemanticPackage) -> Vec<UiLayoutDeclaration> {
    package
        .layout_declarations()
        .map(|layout| layout.declaration().clone())
        .collect()
}

/// A `layout` block seals to the typed declaration it spells: fallback
/// grid, variants in authored order, and an omitted flexible minimum, gap,
/// or padding read as zero.
#[test]
fn a_layout_block_seals_to_the_declaration_it_spells() {
    let package = compile(SOURCE).expect("the layout source seals");

    assert_eq!(sealed_layouts(&package), [declaration()]);
}

/// A layout written in Rust and the same layout written in a `.wui` module
/// are one package.
#[test]
fn rust_and_file_authored_layouts_seal_to_one_package() {
    let file = compile(SOURCE).expect("the layout source seals");
    let rust = WorthUiDslCompiler::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui").with_layout(declaration()),
        ]),
    )
    .expect("the Rust-authored layout seals");

    assert_eq!(sealed_layouts(&rust), sealed_layouts(&file));
    assert_eq!(rust.identity(), file.identity());
}

/// Where the body of `layout demo.component.row { <body> }` begins.
const BODY: usize = "layout demo.component.row { ".len();

/// Each malformed spelling is refused where it stands, and says how the
/// statement reads.
#[test]
fn malformed_layouts_are_source_linked_and_teach_the_statement() {
    for (body, teaches) in [
        ("rows fixed 40;", "declares its columns"),
        ("columns flex 1;", "declares its rows"),
        (
            "columns flex 1; rows fixed 4; gap 1 1; gap 2 2;",
            "declares `gap` once",
        ),
        (
            "columns flex 1; columns flex 2; rows fixed 4;",
            "declares `columns` once",
        ),
        (
            "columns flex 1; rows fixed 4; width from 1 { columns flex 1; rows fixed 4; \
             width from 2 { columns flex 1; rows fixed 4; } }",
            "not inside another variant",
        ),
        (
            "columns flex 1; rows fixed 4; align start;",
            "no `align` statement",
        ),
        ("columns fixed 70000; rows fixed 4;", "from 0 to 65535"),
        ("columns flex; rows fixed 4;", "from 0 to 65535"),
        (
            "columns grow 1; rows fixed 4;",
            "`fixed <extent>` or `flex <weight>",
        ),
        (
            "columns flex 1; rows fixed 4; member demo.component.a 0 0;",
            "member <component> at",
        ),
        (
            "columns flex 1; rows fixed 4; width 600 { }",
            "`width from <min>",
        ),
        ("columns flex 1 rows fixed 4;", "ends with ';'"),
        ("columns flex 1; rows fixed", "from 0 to 65535"),
    ] {
        let source = format!("layout demo.component.row {{ {body} }}");
        let report = compile(&source).expect_err("a malformed layout is refused");
        let [diagnostic] = report.diagnostics() else {
            panic!("one malformed layout produces one diagnostic: {source}")
        };

        assert_eq!(
            diagnostic.identity().code(),
            WorthUiDslCompileDiagnosticCode::InvalidLayoutDeclaration,
            "{source}"
        );
        assert_eq!(
            diagnostic.identity().span().unwrap().module_id(),
            "app/main.wui"
        );
        assert!(
            diagnostic.message().contains(teaches),
            "{source}: {}",
            diagnostic.message()
        );
    }
}

/// A diagnostic points at the statement or token that is wrong, and at the
/// whole declaration only when the body ends before a statement does.
#[test]
fn layout_diagnostics_point_at_what_is_wrong() {
    for (body, start) in [
        ("columns flex 1; rows fixed 4; gap 1 1; gap 2 2;", BODY + 39),
        ("columns flex 1; rows fixed 4; align start;", BODY + 30),
        ("columns fixed 70000; rows fixed 4;", BODY + 14),
        ("columns flex 1; rows fixed", 0),
    ] {
        let source = format!("layout demo.component.row {{ {body} }}");
        let report = compile(&source).expect_err("a malformed layout is refused");
        let [diagnostic] = report.diagnostics() else {
            panic!("one malformed layout produces one diagnostic: {source}")
        };

        assert_eq!(
            diagnostic.identity().span().unwrap().start_byte(),
            start,
            "{source}"
        );
    }
}

/// Points are whole and never negative: a sign or a fraction is not a
/// number the source language reads.
#[test]
fn negative_and_fractional_points_do_not_read() {
    for (track, refused_at) in [
        ("fixed -4", "-"),
        ("fixed 1.5", "."),
        ("flex 1 min -2", "-"),
    ] {
        let source = format!("layout demo.component.row {{ columns {track}; rows fixed 4; }}");
        let report = compile(&source).expect_err("a negative or fractional extent is refused");
        let [diagnostic] = report.diagnostics() else {
            panic!("one unreadable extent produces one diagnostic: {source}")
        };

        assert_eq!(
            diagnostic.identity().code(),
            WorthUiDslCompileDiagnosticCode::InvalidCharacter,
            "{source}"
        );
        assert_eq!(
            diagnostic.identity().span().unwrap().start_byte(),
            source
                .rfind(refused_at)
                .expect("the fixture holds the refused character"),
            "{source}"
        );
    }
}

/// The language refuses only a reference no component could carry; the
/// identity grammar is judged where the layout meets the registry.
#[test]
fn a_layout_names_a_component_reference() {
    let name = "a".repeat(129);
    let report = compile(&format!(
        "layout {name} {{ columns flex 1; rows fixed 4; }}"
    ))
    .expect_err("a container too long to be a component reference is refused");
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed container produces one diagnostic")
    };

    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::InvalidLayoutDeclaration
    );
    assert!(diagnostic.message().contains("is not a component identity"));
}
