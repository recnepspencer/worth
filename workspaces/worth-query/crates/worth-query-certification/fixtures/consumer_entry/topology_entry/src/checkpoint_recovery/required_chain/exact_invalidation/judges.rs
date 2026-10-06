//! What the court reads and judges: every settled output against the model,
//! and every settlement against a fresh read.

use super::*;

impl Court<'_, '_, '_, '_> {
    /// The output read through a settlement is the output a fresh read
    /// finds: no demand settles on a row another demand has republished
    /// since. A clean output reports the commit it was established at, so the
    /// settlement is judged on what it reads and not on where.
    pub(super) fn judge_settlement(
        &self,
        body: &str,
        settled: &worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation,
        at: &str,
    ) {
        let reported = self
            .request
            .at(settled)
            .query(PlanarOutputRead {
                body_key: body.to_owned(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        assert_eq!(
            PositiveLength::get(&reported.rows()[0].value),
            self.length(body),
            "{at}: the settlement of {body} reads the output a fresh read finds"
        );
    }

    pub(super) fn next_idempotency(&self) -> u64 {
        let key = self.idempotency.get();
        self.idempotency.set(key + 1);
        key
    }

    /// The published Length of `body`, read through the output query.
    pub(super) fn length(&self, body: &str) -> u64 {
        let output = self
            .request
            .query(PlanarOutputRead {
                body_key: body.to_owned(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        PositiveLength::get(&output.rows()[0].value)
    }

    /// Demands the ring's root, both chain nodes and the root of its
    /// successor afresh, settles them in dependency order and judges them.
    /// Returns what each cost and how many chain decisions ran.
    pub(super) fn demand_ring(
        &self,
        rings: &mut [Ring],
        index: usize,
        at: &str,
    ) -> ([Cost; 4], usize) {
        let ring = &rings[index];
        let mut a = root!(self, ring.key("a"), at);
        let mut b = consumer!(self, ring.key("b"), at);
        let mut c = consumer!(self, ring.key("c"), at);
        let mut successor = root!(self, ring.successor.clone(), at);
        let costs = [
            settled!(self, a, at),
            settled!(self, b, at),
            settled!(self, c, at),
            settled!(self, successor, at),
        ];
        drop((a, b, c, successor));
        let decisions = judge_decisions(rings, at);
        let ring = &rings[index];
        self.judge_chain(ring, at);
        assert_eq!(
            self.length(&ring.successor),
            ring.successor_y + 1,
            "{at}: {} publishes what the model computes",
            ring.successor
        );
        (costs, decisions)
    }

    /// The settled root and middle consumer publish what the model computes
    /// from scratch, and the consumer's decision read the root's output.
    pub(super) fn judge_middle(&self, ring: &Ring, at: &str) {
        assert_eq!(
            (
                self.length(&ring.key("a")),
                self.length(&ring.key("b")),
                ring.b_read
            ),
            (ring.root_output(), ring.b_length, Some(ring.root_output())),
            "{at}: ring {} settles its root and middle consumer on the model: {ring:?}",
            ring.index
        );
    }

    /// The whole settled chain agrees with the model.
    pub(super) fn judge_chain(&self, ring: &Ring, at: &str) {
        self.judge_middle(ring, at);
        assert_eq!(
            (self.length(&ring.key("c")), ring.c_read),
            (ring.c_length, Some(ring.b_length)),
            "{at}: ring {} settles its last consumer on the model: {ring:?}",
            ring.index
        );
    }
}
