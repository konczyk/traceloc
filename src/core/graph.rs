use crate::core::ids::NodeId;

pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    pub fn new(node_count: usize) -> Self {
        Self {
            graph: Graph::new(node_count),
        }
    }

    pub fn add_edge(&mut self, src: NodeId, dst: NodeId, amount: u64, timestamp: u64) {
        self.graph.srcs.push(src);
        self.graph.dsts.push(dst);
        self.graph.amounts.push(amount);
        self.graph.timestamps.push(timestamp);
    }

    pub fn freeze(mut self) -> Graph {
        if self.graph.edge_count() == 0 {
            return self.graph;
        }

        // store number of edges per source node
        let mut buf = vec![0; self.graph.node_count];
        for src in &self.graph.srcs {
            buf[*src as usize] += 1;
        }

        // compute edge offsets per source node
        let mut next = 0;
        for (i, edges) in buf.iter().enumerate() {
            let from = next;
            let to = from + edges;
            self.graph.offsets[i] = from;
            self.graph.offsets[i + 1] = to;
            next = to;
        }

        buf.fill(0);
        let mut e = 0;
        for _ in 0..self.graph.edge_count() {
            let src = self.graph.srcs[e] as usize;
            let idx = self.graph.offsets[src] + buf[src];
            if idx != e {
                self.graph.srcs.swap(idx, e);
                self.graph.dsts.swap(idx, e);
                self.graph.amounts.swap(idx, e);
                self.graph.timestamps.swap(idx, e);
            } else {
                e += 1;
            }
            buf[src] += 1;
        }

        self.graph
    }
}

pub struct Graph {
    node_count: usize,
    srcs: Vec<NodeId>,
    dsts: Vec<NodeId>,
    amounts: Vec<u64>,
    timestamps: Vec<u64>,
    offsets: Vec<usize>,
}

impl Graph {
    fn new(node_count: usize) -> Self {
        Self {
            node_count,
            srcs: vec![],
            dsts: vec![],
            amounts: vec![],
            timestamps: vec![],
            offsets: vec![0; node_count + 1],
        }
    }

    pub fn edge_count(&self) -> usize {
        self.srcs.len()
    }

    pub fn edges_from(&'_ self, src: NodeId) -> EdgeIter<'_> {
        EdgeIter::new(self, src)
    }
}

pub struct EdgeIter<'a> {
    graph: &'a Graph,
    start: usize,
    end: usize,
    next: usize,
}

impl<'a> EdgeIter<'a> {
    pub fn new(graph: &'a Graph, node_id: NodeId) -> Self {
        Self {
            graph,
            start: graph.offsets[node_id as usize],
            end: graph.offsets[node_id as usize + 1],
            next: 0,
        }
    }
}

impl<'a> Iterator for EdgeIter<'a> {
    type Item = EdgeRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start + self.next < self.end {
            let result = Some(EdgeRef::new(
                self.graph.dsts[self.start + self.next],
                self.graph.amounts[self.start + self.next],
                self.graph.timestamps[self.start + self.next],
            ));
            self.next += 1;
            result
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EdgeRef {
    pub(crate) dst: NodeId,
    amount: u64,
    timestamp: u64,
}

impl EdgeRef {
    pub fn new(dst: NodeId, amount: u64, timestamp: u64) -> Self {
        Self {
            dst,
            amount,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_edges() {
        let gb = GraphBuilder::new(2);
        let g = gb.freeze();

        assert_eq!(3, g.offsets.len());
        assert!(g.offsets.iter().all(|off| *off == 0));
        assert_eq!(0, g.edge_count());
        assert_eq!(0, g.edges_from(0).count());
        assert_eq!(0, g.edges_from(1).count());
    }

    #[test]
    fn test_single_edge() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 3);
        let g = gb.freeze();

        assert_eq!(vec![0, 1, 1], g.offsets);
        assert_eq!(Some(EdgeRef::new(1, 2, 3)), g.edges_from(0).next());
        assert_eq!(None, g.edges_from(1).next());
    }

    #[test]
    fn test_single_source_edges() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 1, 2);
        gb.add_edge(0, 2, 2, 3);
        gb.add_edge(0, 3, 3, 4);
        let g = gb.freeze();

        assert_eq!(vec![0, 3, 3, 3], g.offsets);
        let mut iter = g.edges_from(0);
        assert_eq!(Some(EdgeRef::new(1, 1, 2)), iter.next());
        assert_eq!(Some(EdgeRef::new(2, 2, 3)), iter.next());
        assert_eq!(Some(EdgeRef::new(3, 3, 4)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_multiple_edges() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(2, 0, 1, 2);
        gb.add_edge(0, 1, 3, 4);
        gb.add_edge(1, 2, 5, 6);
        gb.add_edge(0, 2, 7, 8);
        let g = gb.freeze();

        assert_eq!(vec![0, 2, 3, 4], g.offsets);
        let mut iter = g.edges_from(0);
        assert_eq!(Some(EdgeRef::new(2, 7, 8)), iter.next());
        assert_eq!(Some(EdgeRef::new(1, 3, 4)), iter.next());
        assert_eq!(None, iter.next());
        let mut iter = g.edges_from(1);
        assert_eq!(Some(EdgeRef::new(2, 5, 6)), iter.next());
        assert_eq!(None, iter.next());
        let mut iter = g.edges_from(2);
        assert_eq!(Some(EdgeRef::new(0, 1, 2)), iter.next());
        assert_eq!(None, iter.next());
    }
}
