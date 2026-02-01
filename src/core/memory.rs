use crate::ingest::synthetic::SyntheticEdge;

pub struct MemoryStats {
    pub edges: usize,
    pub bytes: usize,
}

pub fn estimate_edge_memory(edges: usize) -> MemoryStats {
    MemoryStats {
        edges,
        bytes: edges * size_of::<SyntheticEdge>(),
    }
}
