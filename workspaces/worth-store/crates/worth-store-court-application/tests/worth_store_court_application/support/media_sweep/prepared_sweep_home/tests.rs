use super::{DirectoryImage, PristineFile};

#[test]
fn differential_restore_matches_pristine_tree_after_arbitrary_mutations() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("home");
    std::fs::create_dir_all(root.join("sub/deep")).unwrap();
    std::fs::create_dir_all(root.join("empty_dir")).unwrap();
    std::fs::create_dir_all(root.join("dir_to_file")).unwrap();
    std::fs::create_dir_all(root.join("ro_dir/inner")).unwrap();

    std::fs::write(
        root.join("unchanged.bin"),
        b"exact-pristine-bytes-0123456789",
    )
    .unwrap();
    std::fs::write(root.join("same_len.bin"), b"abcdefgh").unwrap();
    std::fs::write(root.join("truncate_me.bin"), b"0123456789abcdef").unwrap();
    std::fs::write(root.join("extend_me.bin"), b"short").unwrap();
    std::fs::write(root.join("delete_me.bin"), b"must-come-back").unwrap();
    std::fs::write(root.join("file_to_dir.bin"), b"was-a-file").unwrap();
    std::fs::write(root.join("sub/deep/nested.bin"), b"nested-payload").unwrap();
    std::fs::write(root.join("dir_to_file/child.bin"), b"inside-dir").unwrap();
    std::fs::write(root.join("ro_dir/inner/kept.bin"), b"kept-inside-ro-dir").unwrap();

    let ro_path = root.join("readonly_pristine.bin");
    std::fs::write(&ro_path, b"readonly-pristine-content").unwrap();
    set_readonly(&ro_path, true);
    let ro_dir_inner = root.join("ro_dir/inner");
    let ro_dir = root.join("ro_dir");
    set_readonly(&ro_dir_inner, true);
    set_readonly(&ro_dir, true);

    let pristine = DirectoryImage::read(&root);
    assert_eq!(pristine.files.len(), 10);
    assert!(pristine.files[std::path::Path::new("readonly_pristine.bin")].readonly);
    assert!(pristine.directories[std::path::Path::new("ro_dir")]);
    assert!(pristine.directories[std::path::Path::new("ro_dir/inner")]);

    // 1. Extra files (writable + read-only) and extra nested read-only directory.
    std::fs::write(root.join("extra_root.tmp"), b"garbage").unwrap();
    let extra_ro = root.join("sub/extra_ro.tmp");
    std::fs::write(&extra_ro, b"ro-garbage").unwrap();
    set_readonly(&extra_ro, true);

    let extra_nested_dir = root.join("extra_dir/nested");
    std::fs::create_dir_all(&extra_nested_dir).unwrap();
    let extra_nested = extra_nested_dir.join("ro_leaf.dat");
    std::fs::write(&extra_nested, b"nested-garbage").unwrap();
    set_readonly(&extra_nested, true);
    set_readonly(&extra_nested_dir, true);

    // Clear read-only on ro_dir to plant an intruder file inside it, then restore ro flag.
    set_readonly(&ro_dir, false);
    set_readonly(&ro_dir_inner, false);
    std::fs::write(ro_dir_inner.join("intruder.tmp"), b"extra-inside-ro").unwrap();
    set_readonly(&ro_dir_inner, true);
    set_readonly(&ro_dir, true);

    // 2. Delete pristine file and empty directory.
    std::fs::remove_file(root.join("delete_me.bin")).unwrap();
    std::fs::remove_dir(root.join("empty_dir")).unwrap();

    // 3. Same-length byte flip, truncation, and extension.
    std::fs::write(root.join("same_len.bin"), b"abcdEFGH").unwrap();
    std::fs::write(root.join("truncate_me.bin"), b"0123").unwrap();
    std::fs::write(root.join("extend_me.bin"), b"short-extended-much-longer").unwrap();

    // 4. Type swaps: file -> dir and dir -> file.
    std::fs::remove_file(root.join("file_to_dir.bin")).unwrap();
    std::fs::create_dir(root.join("file_to_dir.bin")).unwrap();
    std::fs::write(root.join("file_to_dir.bin/intruder"), b"x").unwrap();

    std::fs::remove_dir_all(root.join("dir_to_file")).unwrap();
    std::fs::write(root.join("dir_to_file"), b"now-a-file").unwrap();

    // 5. Flip permissions on both writable and read-only pristine files.
    set_readonly(&ro_path, false);
    std::fs::write(&ro_path, b"corrupted-readonly").unwrap();
    set_readonly(&root.join("sub/deep/nested.bin"), true);

    // 6. Directory link (junction on Windows, symlink_dir on Unix) pointing outside root:
    //    must be unlinked cleanly without modifying or deleting target contents.
    let external = tmp.path().join("external_target");
    std::fs::create_dir_all(&external).unwrap();
    let sentinel = external.join("sentinel.txt");
    std::fs::write(&sentinel, b"must-stay-untouched").unwrap();
    let dir_link = root.join("dir_link_intruder");
    create_dir_link(&dir_link, &external);

    // Restore differentially.
    let stats = pristine.restore_differential(&root);
    assert!(
        stats.unchanged_files >= 3,
        "unchanged files should not be rewritten: {stats:?}"
    );
    assert_eq!(
        stats.rewritten_files, 7,
        "exact count of mutated/missing files: {stats:?}"
    );
    assert_eq!(
        stats.metadata_fixed, 1,
        "nested.bin readonly flag should be fixed without rewrite"
    );
    assert!(
        stats.removed_entries >= 6,
        "extra entries, links, and type swaps should be pruned: {stats:?}"
    );

    // Verify external directory link target remained completely untouched.
    assert!(!dir_link.exists(), "directory link must be unlinked");
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"must-stay-untouched",
        "external directory link target must remain untouched"
    );

    // Verify 100% byte-for-byte, directory-for-directory, and permission parity.
    let restored = DirectoryImage::read(&root);
    assert_eq!(restored, pristine);
    assert_eq!(
        restored
            .files
            .get(std::path::Path::new("readonly_pristine.bin")),
        Some(&PristineFile {
            bytes: b"readonly-pristine-content".as_slice().into(),
            readonly: true,
        })
    );

    // A second restore on an already-clean tree rewrites zero files.
    let second = pristine.restore_differential(&root);
    assert_eq!(second.rewritten_files, 0);
    assert_eq!(second.removed_entries, 0);
    assert_eq!(second.created_dirs, 0);
    assert_eq!(second.metadata_fixed, 0);
    assert_eq!(second.unchanged_files, 10);

    // Clear read-only flags before TempDir cleanup on Windows.
    set_readonly(&ro_dir, false);
    set_readonly(&ro_dir_inner, false);
    set_readonly(&ro_path, false);
}

fn set_readonly(path: &std::path::Path, readonly: bool) {
    let mut perms = std::fs::symlink_metadata(path).unwrap().permissions();
    perms.set_readonly(readonly);
    std::fs::set_permissions(path, perms).unwrap();
}

fn create_dir_link(link: &std::path::Path, target: &std::path::Path) {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return;
        }
        // NTFS directory junctions require no elevated privilege on Windows
        // and present as `is_symlink() && is_symlink_dir()` reparse points.
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()
            .expect("invoke mklink /J");
        assert!(status.success(), "mklink /J failed with {status}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).expect("create directory symlink");
    }
}
