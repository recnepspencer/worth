//! Independent wire oracle for the v3 canonical incidence tree.
use sha2::{Digest, Sha256};
fn sha(domain: &[u8], chunks: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for chunk in chunks {
        hash.update(chunk);
    }
    hash.finalize().into()
}
fn tree(rows: &[(u128, [u8; 32])]) -> [u8; 32] {
    const DOMAIN: &[u8] = b"worth.relational.content-index.v1\0";
    match rows {
        [] => sha(DOMAIN, &[&[0]]),
        [(key, value)] => sha(DOMAIN, &[&[1], &key.to_be_bytes(), value]),
        _ => {
            let bit = (rows.first().unwrap().0 ^ rows.last().unwrap().0).leading_zeros();
            let split = rows
                .iter()
                .position(|(key, _)| (key >> (127 - bit)) & 1 == 1)
                .unwrap();
            sha(
                DOMAIN,
                &[
                    &[2],
                    &bit.to_be_bytes(),
                    &tree(&rows[..split]),
                    &tree(&rows[split..]),
                ],
            )
        }
    }
}
pub(super) fn digest(
    projection: &[crate::storage::partition::RegionRelationProjection],
) -> [u8; 32] {
    let entries: Vec<_> = projection
        .iter()
        .map(|entry| {
            let slots = |values: &std::collections::BTreeSet<usize>| {
                let rows: Vec<_> = values
                    .iter()
                    .map(|slot| {
                        (
                            *slot as u128,
                            sha(
                                b"worth.relational.region-kind-slot.v1\0",
                                &[&(*slot as u64).to_be_bytes()],
                            ),
                        )
                    })
                    .collect();
                tree(&rows)
            };
            let incidence = sha(
                b"worth.relational.region-kind-incidence.v1\0",
                &[
                    &entry.kind.0.to_be_bytes(),
                    &slots(&entry.outgoing),
                    &slots(&entry.incoming),
                ],
            );
            let id = entry.relation;
            let key = ((id.partition_id.as_u32() as u128) << 96)
                | ((id.slot_index() as u128) << 32)
                | id.generation_value() as u128;
            (key, incidence)
        })
        .collect();
    sha(
        b"worth.relational.region-kind-authority.v3\0",
        &[&tree(&entries)],
    )
}
