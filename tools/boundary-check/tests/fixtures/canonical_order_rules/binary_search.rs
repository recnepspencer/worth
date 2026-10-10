fn decide(values: &[Token], type_id: Token) {
    values.binary_search(&type_id);
}
