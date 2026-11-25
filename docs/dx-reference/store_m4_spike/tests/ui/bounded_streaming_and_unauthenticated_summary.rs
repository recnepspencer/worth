use store_m4_dx_spike::*;

fn unbounded_stream_and_raw_summary_denied(mut reader: ValueReader<'_>) {
    let _all: Vec<u8> = reader.read_all_to_vec();
    let raw = ChunkKindAdjacencySummary { kind_bits: 0 };
    let _skip = evaluate_cold_chunk_skip(&raw, 1);
}

fn main() {}
