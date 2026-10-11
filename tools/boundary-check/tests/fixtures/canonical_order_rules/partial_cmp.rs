fn decide(left: Token, type_id: Token) {
    left.partial_cmp(&type_id);
}
