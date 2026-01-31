use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct SyntheticConfig {
    pub node_count: u32,
    pub edge_count: u64,
    pub seed: u64,
}

pub struct SyntheticEdge {
    pub src: u32,
    pub dst: u32,
    pub amount: u64,
    pub timestamp: u64,
}

pub fn generate(cfg: &SyntheticConfig) -> impl Iterator<Item = SyntheticEdge> {
    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let node_count = cfg.node_count;
    let edge_count = cfg.edge_count;

    (0..edge_count).map(move |_| {
        let src = rng.random_range(0..node_count);
        let mut dst = rng.random_range(0..node_count);
        if dst == src {
            dst = (dst + 1) % node_count;
        }

        SyntheticEdge {
            src,
            dst,
            amount: rng.random_range(1_000..100_000),
            timestamp: rng.random_range(1_600_000_000..1_700_000_000),
        }
    })
}
