use worth_proof::{DisjointPair, ExactlyOne, Pair};

fn takes_exactly_one(_: ExactlyOne<u8>) {}

fn takes_pair(_: Pair<u8>) {}

fn takes_disjoint_pair(_: DisjointPair<u8>) {}

fn main() {
    let scalar = 7_u8;
    let vec = vec![1_u8, 2_u8];
    let array = [1_u8, 2_u8];
    let pair = Pair::new(1_u8, 2_u8);

    takes_exactly_one(scalar);
    takes_pair(vec);
    takes_disjoint_pair(array);
    takes_disjoint_pair(pair);
}
