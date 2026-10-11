fn decide(mut entries: Vec<Entry>, names: Names) {
    entries.sort_by_key(|e| names.identity(e.type_id));
}
