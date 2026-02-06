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
        self.graph.srcs_out.push(src);
        self.graph.dsts.push(dst);
        self.graph.amounts.push(amount);
        self.graph.timestamps.push(timestamp);
    }

    pub fn freeze(mut self) -> Graph {
        if self.graph.edge_count() == 0 {
            return self.graph;
        }

        let mut buf = vec![0; self.graph.node_count];

        // store number of edges per dst node
        for dst in &self.graph.dsts {
            buf[*dst as usize] += 1;
        }

        // compute edge offsets per dst node
        let mut next = 0;
        for (i, edges) in buf.iter().enumerate() {
            let from = next;
            let to = from + edges;
            self.graph.offsets_in[i] = from;
            self.graph.offsets_in[i + 1] = to;
            next = to;
        }

        buf.fill(0);
        self.graph.srcs_in = vec![0; self.graph.edge_count()];
        for e in 0..self.graph.edge_count() {
            let dst = self.graph.dsts[e] as usize;
            let idx = self.graph.offsets_in[dst] + buf[dst];
            self.graph.srcs_in[idx] = self.graph.srcs_out[e];
            buf[dst] += 1;
        }

        buf.fill(0);
        // store number of edges per src node
        for src in &self.graph.srcs_out {
            buf[*src as usize] += 1;
        }

        // compute edge offsets per source node
        next = 0;
        for (i, edges) in buf.iter().enumerate() {
            let from = next;
            let to = from + edges;
            self.graph.offsets_out[i] = from;
            self.graph.offsets_out[i + 1] = to;
            next = to;
        }

        buf.fill(0);
        let mut e = 0;
        for _ in 0..self.graph.edge_count() {
            let src = self.graph.srcs_out[e] as usize;
            let idx = self.graph.offsets_out[src] + buf[src];
            if idx != e {
                self.graph.srcs_out.swap(idx, e);
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
    srcs_out: Vec<NodeId>,
    srcs_in: Vec<NodeId>,
    dsts: Vec<NodeId>,
    amounts: Vec<u64>,
    timestamps: Vec<u64>,
    offsets_out: Vec<usize>,
    offsets_in: Vec<usize>,
}

impl Graph {
    fn new(node_count: usize) -> Self {
        Self {
            node_count,
            srcs_out: vec![],
            srcs_in: vec![],
            dsts: vec![],
            amounts: vec![],
            timestamps: vec![],
            offsets_out: vec![0; node_count + 1],
            offsets_in: vec![0; node_count + 1],
        }
    }

    pub fn edge_count(&self) -> usize {
        self.srcs_out.len()
    }

    pub fn edges_from(&'_ self, src: NodeId) -> OutgoingEdgeIter<'_> {
        OutgoingEdgeIter::new(self, src)
    }

    pub fn edges_to(&'_ self, dst: NodeId) -> IncomingEdgeIter<'_> {
        IncomingEdgeIter::new(self, dst)
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

pub struct IncomingEdgeIter<'a> {
    graph: &'a Graph,
    start: usize,
    end: usize,
    next: usize,
}

impl<'a> IncomingEdgeIter<'a> {
    pub fn new(graph: &'a Graph, node_id: NodeId) -> Self {
        Self {
            graph,
            start: graph.offsets_in[node_id as usize],
            end: graph.offsets_in[node_id as usize + 1],
            next: 0,
        }
    }
}

impl<'a> Iterator for IncomingEdgeIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start + self.next < self.end {
            let result = Some(self.graph.srcs_in[self.start + self.next]);
            self.next += 1;
            result
        } else {
            None
        }
    }
}

pub struct OutgoingEdgeIter<'a> {
    graph: &'a Graph,
    start: usize,
    end: usize,
    next: usize,
}

impl<'a> OutgoingEdgeIter<'a> {
    pub fn new(graph: &'a Graph, node_id: NodeId) -> Self {
        Self {
            graph,
            start: graph.offsets_out[node_id as usize],
            end: graph.offsets_out[node_id as usize + 1],
            next: 0,
        }
    }
}

impl<'a> Iterator for OutgoingEdgeIter<'a> {
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
    pub dst: NodeId,
    pub amount: u64,
    pub timestamp: u64,
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

        assert_eq!(3, g.offsets_out.len());
        assert!(g.offsets_out.iter().all(|off| *off == 0));
        assert_eq!(0, g.edge_count());
        assert_eq!(0, g.edges_from(0).count());
        assert_eq!(0, g.edges_from(1).count());
        assert_eq!(0, g.edges_to(0).count());
        assert_eq!(0, g.edges_to(1).count());
    }

    #[test]
    fn test_single_edge() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 3);
        let g = gb.freeze();

        assert_eq!(vec![0, 1, 1], g.offsets_out);
        assert_eq!(Some(EdgeRef::new(1, 2, 3)), g.edges_from(0).next());
        assert_eq!(None, g.edges_from(1).next());

        assert_eq!(Some(0), g.edges_to(1).next());
        assert_eq!(None, g.edges_to(0).next());
    }

    #[test]
    fn test_single_source_edges() {
        let mut gb = GraphBuilder::new(4);
        gb.add_edge(0, 1, 1, 2);
        gb.add_edge(0, 2, 2, 3);
        gb.add_edge(0, 3, 3, 4);
        let g = gb.freeze();

        assert_eq!(vec![0, 3, 3, 3, 3], g.offsets_out);
        let mut iter = g.edges_from(0);
        assert_eq!(Some(EdgeRef::new(1, 1, 2)), iter.next());
        assert_eq!(Some(EdgeRef::new(2, 2, 3)), iter.next());
        assert_eq!(Some(EdgeRef::new(3, 3, 4)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_single_destination_edges() {
        let mut gb = GraphBuilder::new(4);
        gb.add_edge(1, 0, 1, 2);
        gb.add_edge(2, 0, 2, 3);
        gb.add_edge(3, 0, 3, 4);
        let g = gb.freeze();

        assert_eq!(vec![0, 3, 3, 3, 3], g.offsets_in);
        let mut iter = g.edges_to(0);
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(Some(3), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_multiple_edges() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 2, 7, 8);
        gb.add_edge(2, 0, 1, 2);
        gb.add_edge(0, 1, 3, 4);
        gb.add_edge(1, 2, 5, 6);
        let g = gb.freeze();

        assert_eq!(vec![0, 2, 3, 4], g.offsets_out);
        assert_eq!(vec![0, 1, 2, 4], g.offsets_in);
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

        let mut iter = g.edges_to(0);
        assert_eq!(Some(2), iter.next());
        assert_eq!(None, iter.next());
        let mut iter = g.edges_to(1);
        assert_eq!(Some(0), iter.next());
        assert_eq!(None, iter.next());
        let mut iter = g.edges_to(2);
        assert_eq!(Some(0), iter.next());
        assert_eq!(Some(1), iter.next());
        assert_eq!(None, iter.next());
    }
}
