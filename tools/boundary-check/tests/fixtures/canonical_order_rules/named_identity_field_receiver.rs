fn decide(mut entries: Vec<Entry>, receiver: Receiver) {
    entries.sort_by_key(|e| receiver.names.identity(e.type_id));
}
