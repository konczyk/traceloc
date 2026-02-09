use crate::analysis::mixer::{DEG_THRESHOLD, WINDOW_SIZE};
use crate::core::graph::{Graph, GraphBuilder};
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

pub fn star_graph(node_count: u32) -> Graph {
    assert!(node_count >= 2);

    let mut gb = GraphBuilder::new(node_count as usize);

    for leaf in 1..node_count {
        gb.add_edge(0, leaf, 1, 0);
        gb.add_edge(leaf, 0, 1, 0);
    }

    gb.freeze()
}

pub fn normal_user_graph() -> Graph {
    let mut gb = GraphBuilder::new(3);
    gb.add_edge(0, 1, 1, 0);
    gb.add_edge(2, 0, 1, WINDOW_SIZE * 2);
    gb.freeze()
}

pub fn bridge_node_graph() -> Graph {
    let mut gb = GraphBuilder::new(7);
    // Group A
    gb.add_edge(1, 2, 1, 0);
    gb.add_edge(2, 3, 1, 0);
    // Group B
    gb.add_edge(4, 5, 1, 0);
    gb.add_edge(5, 6, 1, 0);
    // Bridge node = 0
    gb.add_edge(3, 0, 1, 100);
    gb.add_edge(0, 4, 1, 100 + WINDOW_SIZE / 2);
    gb.freeze()
}

pub fn exchange_hub_graph() -> Graph {
    let mut gb = GraphBuilder::new(((DEG_THRESHOLD as usize + 2) * 2) + 1);
    for i in 1..=DEG_THRESHOLD + 2 {
        gb.add_edge(i, 0, 1, 0);
    }
    for i in DEG_THRESHOLD + 3..=(DEG_THRESHOLD + 2) * 2 {
        gb.add_edge(0, i, 1, WINDOW_SIZE * 10);
    }
    gb.freeze()
}

pub fn strong_mixer_graph() -> Graph {
    let mut gb = GraphBuilder::new(DEG_THRESHOLD as usize * 3 + 1);

    for i in 1..=DEG_THRESHOLD {
        gb.add_edge(i, 0, 1, 100);
    }
    for i in DEG_THRESHOLD + 1..=DEG_THRESHOLD * 2 {
        gb.add_edge(i, 0, 1, 100);
    }
    for i in DEG_THRESHOLD * 2 + 1..=DEG_THRESHOLD * 3 {
        gb.add_edge(i, 0, 1, 100);
    }

    for i in 1..=DEG_THRESHOLD {
        gb.add_edge(0, i, 1, 100 + WINDOW_SIZE / 2);
    }
    for i in DEG_THRESHOLD + 1..=DEG_THRESHOLD * 2 {
        gb.add_edge(0, i, 1, 100 + WINDOW_SIZE / 2);
    }
    for i in DEG_THRESHOLD * 2 + 1..=DEG_THRESHOLD * 3 {
        gb.add_edge(0, i, 1, 100 + WINDOW_SIZE / 2);
    }

    gb.freeze()
}
