use crate::core::memory::estimate_edge_memory;
use crate::ingest::synthetic::{SyntheticConfig, generate};

pub mod analysis;
pub mod core;
pub mod ingest;

fn main() {
    let cfg = SyntheticConfig {
        node_count: 1_000_000,
        edge_count: 10_000_000,
        seed: 42,
    };

    let edge_count = generate(&cfg).count();

    let stats = estimate_edge_memory(edge_count);

    println!(
        "edges: {}, approx memory: {} MB",
        stats.edges,
        stats.bytes / (1024 * 1024)
    );
}
