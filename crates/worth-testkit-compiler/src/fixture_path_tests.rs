//! File-pattern partitions retain every matching compiler input.
use crate::paths;

#[test]
fn character_class_partitions_select_each_source_exactly_once() {
    let directory = std::env::temp_dir().join(format!(
        "worth-compiler-pattern-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let names = [
        "alpha.rs",
        "foxtrot.rs",
        "golf.rs",
        "romeo.rs",
        "sierra.rs",
        "zulu.rs",
    ];
    for name in names {
        std::fs::write(directory.join(name), "fn main() {}\n").unwrap();
    }
    std::fs::create_dir(directory.join("folder.rs")).unwrap();
    let mut selected = Vec::new();
    for pattern in ["[a-f]*.rs", "[g-r]*.rs", "[s-z]*.rs"] {
        let expanded = paths::expand(&directory.join(pattern)).unwrap();
        assert_eq!(expanded.len(), 2);
        selected.extend(expanded);
    }
    assert_eq!(selected, names.map(|name| directory.join(name)));
    assert_eq!(paths::expand(&directory.join("*.rs")).unwrap(), selected);
    for name in names {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
    std::fs::remove_dir(directory.join("folder.rs")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
