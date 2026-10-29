#[allow(unused_imports)]
pub(crate) use crate::fast::expressions::reference::*;

#[test]
fn generated_integer_and_boolean_trees_match_the_reference() {
    let mut random = Seeded(0x0DD5_EED5_2026_0928);
    let (mut denied, mut values) = (0, 0);
    for _ in 0..600 {
        let depth = 1 + random.below(4) as u32;
        if random.below(2) == 0 {
            let tree = random.int(depth);
            let expected = int(&tree).map(ExpressionValue::integer);
            *(if expected.is_ok() {
                &mut values
            } else {
                &mut denied
            }) += 1;
            check(&render_int(&tree), expected);
        } else {
            let tree = random.boolean(depth);
            let expected = boolean(&tree).map(ExpressionValue::bool);
            *(if expected.is_ok() {
                &mut values
            } else {
                &mut denied
            }) += 1;
            check(&render_bool(&tree), expected);
        }
    }
    assert!(
        values > 200 && denied > 50,
        "the corpus covers both ({values}, {denied})"
    );
}
