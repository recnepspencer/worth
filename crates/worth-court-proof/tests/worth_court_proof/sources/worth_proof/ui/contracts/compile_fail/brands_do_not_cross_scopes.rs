use worth_proof::Branded;

fn require_same_scope<'id>(_: Branded<'id, u8>, _: Branded<'id, u8>) {}

fn main() {
    worth_proof::with_brand(|left| {
        worth_proof::with_brand(|right| {
            require_same_scope(left.bind(1), right.bind(2));
        });
    });
}
