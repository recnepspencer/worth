//! Named edit shapes and their literal call laws, derived before execution.
use super::*;
pub(in super::super::super) struct LawCase {
    pub name: &'static str,
    pub before: Model,
    pub after: Model,
    pub changes: Vec<Change>,
    pub calls: [usize; 4],
    pub stopped_prior: bool,
    pub own_write: Option<OwnWrite>,
}
impl Model {
    pub(in super::super::super) fn law_cases() -> Vec<LawCase> {
        let mut base = Self::bounded(&mut Lcg(1), 8, 16, TOTALS_WORK);
        for entry in base.entries.iter_mut().flatten() {
            entry.work = 1;
            entry.value = 2.0;
            entry.region %= 2;
        }
        base.weights = [0.0; 2];
        let mut cases = Vec::new();
        let own = {
            let mut push = |name, after, changes, calls| {
                cases.push(LawCase {
                    name,
                    before: base.clone(),
                    after,
                    changes,
                    calls,
                    stopped_prior: false,
                    own_write: None,
                })
            };
            let mut now = base.clone();
            assert!(now.regions().contains(&1) && now.entries[8].is_none());
            let change = now.created(8, 1, 3.0, 1);
            push("Create into existing key", now, vec![change], [1, 1, 1, 1]);
            let mut now = base.clone();
            assert!(now.entries[1].unwrap().region == now.entries[3].unwrap().region);
            let change = now.deleted(1);
            push("Delete, key survives", now, vec![change], [1, 0, 1, 1]);
            let mut now = base.clone();
            now.entries[1].as_mut().unwrap().value = 3.0;
            let value = Change::Entry(EntryEdit::new(
                "even",
                1,
                EntryFact::Value,
                3.0_f64.to_bits(),
            ));
            push(
                "Value change",
                now.clone(),
                vec![value.clone()],
                [0, 0, 1, 1],
            );
            let own = LawCase {
                name: "Own-write after value edit",
                before: base.clone(),
                after: now,
                changes: vec![value],
                calls: [0, 0, 1, 1],
                stopped_prior: false,
                own_write: Some(OwnWrite {
                    number: 3,
                    bits: 4.0_f64.to_bits(),
                    read: OwnWriteRead::Observed,
                }),
            };
            assert!(own.after.entries[1].unwrap().region == own.after.entries[3].unwrap().region);
            let mut now = base.clone();
            assert_ne!(now.entries[1].unwrap().region, 0);
            assert!(now.entries[3].unwrap().region == 1 && now.entries[2].unwrap().region == 0);
            let change = now.moved(1, 0);
            push(
                "Key move, both keys survive",
                now,
                vec![change],
                [0, 1, 2, 2],
            );
            let mut now = base.clone();
            assert_ne!(
                now.entries[1].unwrap().region,
                now.entries[2].unwrap().region
            );
            now.entries.swap(1, 2);
            push(
                "Swap across keys",
                now,
                vec![Change::Entry(EntryEdit::swap(1, 2))],
                [1, 2, 2, 2],
            );
            let mut now = base.clone();
            let members = now
                .held(false)
                .into_iter()
                .filter(|n| now.entries[*n].unwrap().region == 0)
                .collect::<Vec<_>>();
            let n = members.len();
            let changes = members.into_iter().map(|n| now.moved(n, 1)).collect();
            assert_eq!(now.regions(), BTreeSet::from([1]));
            push(
                "Empty key by moving its members",
                now,
                changes,
                [0, n, 1, 1],
            );
            let mut now = base.clone();
            assert!(!now.regions().contains(&2));
            let change = now.moved(1, 2);
            assert!(now.regions().contains(&1));
            push("New key, source survives", now, vec![change], [0, 1, 2, 2]);
            let mut now = base.clone();
            now.entries[1].as_mut().unwrap().value = 3.0;
            now.entries[3].as_mut().unwrap().value = 1.0;
            assert_eq!(base.partition_values(), now.partition_values());
            push(
                "Equal-result republication",
                now,
                vec![
                    Change::Entry(EntryEdit::new(
                        "even",
                        1,
                        EntryFact::Value,
                        3.0_f64.to_bits(),
                    )),
                    Change::Entry(EntryEdit::new(
                        "even",
                        3,
                        EntryFact::Value,
                        1.0_f64.to_bits(),
                    )),
                ],
                [0, 0, 1, 1],
            );
            push(
                "Unchanged fork or switch basis",
                base.clone(),
                vec![],
                [0, 0, 0, 0],
            );
            own
        };
        cases.push(own);
        let mut before = base.clone();
        before.entries[0].as_mut().unwrap().work = before.heavy;
        let mut after = before.clone();
        let change = after.created(8, 1, 2.0, after.heavy);
        assert!(before.completes() && !after.completes());
        cases.push(LawCase {
            name: "Work ceiling",
            before: before.clone(),
            after: after.clone(),
            changes: vec![change],
            calls: [1, 1, 1, 1],
            stopped_prior: false,
            own_write: None,
        });
        let change = after.deleted(8);
        assert!(after == before);
        cases.push(LawCase {
            name: "Relief after stopped publication",
            before: {
                let mut stopped = before.clone();
                stopped.created(8, 1, 2.0, stopped.heavy);
                stopped
            },
            after: before.clone(),
            changes: vec![change],
            calls: [1, before.len(), 2, 2],
            stopped_prior: true,
            own_write: None,
        });
        cases
    }
}
