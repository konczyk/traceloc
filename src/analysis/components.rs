use crate::core::graph::Graph;
use std::collections::HashMap;

struct DisjointSet {
    parent: Vec<u32>,
    size: Vec<u32>,
}

impl DisjointSet {
    pub fn new(size: usize) -> Self {
        Self {
            parent: (0..size as u32).collect(),
            size: vec![1; size],
        }
    }

    fn find(&mut self, u: u32) -> u32 {
        let mut ru = u;
        while ru != self.parent[ru as usize] {
            ru = self.parent[ru as usize];
        }
        let mut v = u;
        while v != self.parent[v as usize] {
            let w = v;
            v = self.parent[v as usize];
            self.parent[w as usize] = ru;
        }
        ru
    }

    fn union(&mut self, u: u32, v: u32) {
        let ru = self.find(u) as usize;
        let rv = self.find(v) as usize;
        if ru == rv {
            return;
        }
        if self.size[ru] > self.size[rv] {
            self.parent[rv] = ru as u32;
            self.size[ru] += self.size[rv];
        } else {
            self.parent[ru] = rv as u32;
            self.size[rv] += self.size[ru];
        }
    }
}

pub fn connected_components(graph: &Graph) -> Vec<u32> {
    let mut clusters = HashMap::new();
    let mut dsu = DisjointSet::new(graph.node_count());
    for u in 0..graph.node_count() as u32 {
        for e in graph.edges_from(u) {
            dsu.union(u, e.dst);
        }
    }
    for u in 0..graph.node_count() as u32 {
        let ru = dsu.find(u);
        if !clusters.contains_key(&ru) {
            let clusters_count = clusters.len() as u32;
            clusters.insert(ru, clusters_count);
        }
    }
    let mut result = vec![0; graph.node_count()];
    for u in 0..graph.node_count() as u32 {
        let ru = dsu.find(u);
        let cluster_id = *clusters.get(&ru).unwrap();
        result[u as usize] = cluster_id;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::GraphBuilder;

    #[test]
    fn test_single_component() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 1, 2);
        gb.add_edge(1, 2, 2, 3);
        let g = gb.freeze();

        let cc = connected_components(&g);
        assert_eq!(cc[0], 0);
        assert_eq!(cc[1], 0);
        assert_eq!(cc[2], 0);
    }

    #[test]
    fn test_two_components() {
        let mut gb = GraphBuilder::new(4);
        gb.add_edge(0, 1, 1, 2);
        gb.add_edge(2, 3, 2, 3);
        let g = gb.freeze();

        let cc = connected_components(&g);
        assert_eq!(cc[0], 0);
        assert_eq!(cc[1], 0);
        assert_eq!(cc[2], 1);
        assert_eq!(cc[3], 1);
    }

    #[test]
    fn test_isolated_component() {
        let gb = GraphBuilder::new(1);
        let g = gb.freeze();

        let cc = connected_components(&g);
        assert_eq!(cc[0], 0);
    }

    #[test]
    fn test_path_compression() {
        let mut gb = GraphBuilder::new(5);
        gb.add_edge(0, 1, 0, 0);
        gb.add_edge(1, 2, 0, 0);
        gb.add_edge(2, 3, 0, 0);
        gb.add_edge(3, 4, 0, 0);
        let g = gb.freeze();

        let cc = connected_components(&g);

        let root = cc[0];
        for &id in &cc {
            assert_eq!(id, root);
        }
    }
}
