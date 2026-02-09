use crate::core::graph::Graph;
use crate::core::ids::NodeId;

pub struct DegreeStats {
    pub in_deg: u32,
    pub out_deg: u32,
}

pub fn compute_degree_stats(graph: &Graph) -> Vec<DegreeStats> {
    let mut result = Vec::with_capacity(graph.node_count());
    for node_id in 0..graph.node_count() {
        result.push(DegreeStats {
            in_deg: graph.in_degree(node_id as u32) as u32,
            out_deg: graph.out_degree(node_id as u32) as u32,
        })
    }
    result
}

pub fn compute_label_diversity(graph: &Graph, labels: &[u32]) -> Vec<u32> {
    let mut counts = vec![0; graph.node_count()];
    let mut buf = vec![0; graph.node_count()];
    for n in 0..graph.node_count() {
        for label in graph
            .edges_from(n as u32)
            .map(|e| e.dst)
            .chain(graph.edges_to(n as u32).map(|e| e.src))
            .map(|node_id| labels[node_id as usize])
        {
            if buf[label as usize] < n + 1 {
                counts[n] += 1;
                buf[label as usize] = n + 1;
            }
        }
    }
    counts
}

pub fn has_in_out_overlap(graph: &Graph, node: NodeId, dt: u64) -> bool {
    let mut in_time = graph
        .edges_to(node)
        .map(|e| e.timestamp)
        .collect::<Vec<u64>>();
    let mut out_time = graph
        .edges_from(node)
        .map(|e| e.timestamp)
        .collect::<Vec<u64>>();
    if in_time.is_empty() || out_time.is_empty() {
        return false;
    }

    in_time.sort();
    out_time.sort();

    let mut from = 0;
    for i in 0..in_time.len() {
        for j in from..out_time.len() {
            if out_time[j].abs_diff(in_time[i]) <= dt {
                return true;
            }
            if out_time[j] > in_time[i] + dt {
                from = j;
                break;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::GraphBuilder;
    use crate::ingest::synthetic;

    #[test]
    fn test_no_edges_degree_stats() {
        let gb = GraphBuilder::new(2);
        let g = gb.freeze();

        let stats = compute_degree_stats(&g);
        assert_eq!(2, stats.len());
        assert_eq!(0, stats[0].in_deg);
        assert_eq!(0, stats[1].in_deg);
        assert_eq!(0, stats[0].out_deg);
        assert_eq!(0, stats[1].out_deg);
    }

    #[test]
    fn test_star_graph_degree_stats() {
        let g = synthetic::star_graph(6);

        let stats = compute_degree_stats(&g);
        assert_eq!(6, stats.len());
        assert_eq!(5, stats[0].in_deg);
        assert_eq!(5, stats[0].out_deg);
        for i in 1..6 {
            assert_eq!(1, stats[i].in_deg);
            assert_eq!(1, stats[i].out_deg);
        }
    }

    #[test]
    fn test_no_edges_no_in_out_overlap() {
        let gb = GraphBuilder::new(2);
        let g = gb.freeze();

        assert!(!has_in_out_overlap(&g, 0, 1));
        assert!(!has_in_out_overlap(&g, 1, 1));
    }

    #[test]
    fn test_single_edge_in_out_overlap() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 10);
        gb.add_edge(2, 0, 2, 0);
        let g = gb.freeze();

        assert!(has_in_out_overlap(&g, 0, 10));
    }

    #[test]
    fn test_single_edge_no_in_out_overlap() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 11);
        gb.add_edge(2, 0, 2, 0);
        let g = gb.freeze();

        assert!(!has_in_out_overlap(&g, 0, 10));
    }

    #[test]
    fn test_out_before_in() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 100);
        gb.add_edge(2, 0, 2, 120);
        let g = gb.freeze();

        assert!(has_in_out_overlap(&g, 0, 20));
    }

    #[test]
    fn test_multiple_edges_single_overlap() {
        let mut gb = GraphBuilder::new(6);
        gb.add_edge(0, 1, 2, 0);
        gb.add_edge(0, 2, 2, 1000);
        gb.add_edge(0, 3, 2, 2000);
        gb.add_edge(4, 0, 2, 5000);
        gb.add_edge(5, 0, 2, 1005);
        let g = gb.freeze();

        assert!(has_in_out_overlap(&g, 0, 10));
    }

    #[test]
    fn test_multiple_edges_no_overlap() {
        let mut gb = GraphBuilder::new(6);
        gb.add_edge(0, 1, 2, 0);
        gb.add_edge(0, 2, 2, 1000);
        gb.add_edge(0, 3, 2, 2000);
        gb.add_edge(4, 0, 2, 5000);
        gb.add_edge(5, 0, 2, 3000);
        let g = gb.freeze();

        assert!(!has_in_out_overlap(&g, 0, 100));
    }

    #[test]
    fn test_no_edges_label_diversity() {
        let gb = GraphBuilder::new(2);
        let g = gb.freeze();

        assert_eq!(vec![0, 0], compute_label_diversity(&g, &[0, 1]));
    }

    #[test]
    fn test_single_label_neighborhood() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 0);
        gb.add_edge(0, 2, 2, 0);
        let g = gb.freeze();

        let diversity = compute_label_diversity(&g, &[0, 1, 1]);
        assert_eq!(1, diversity[0]);
    }

    #[test]
    fn test_multi_label_neighborhood() {
        let mut gb = GraphBuilder::new(4);
        gb.add_edge(0, 1, 2, 0);
        gb.add_edge(0, 2, 2, 0);
        gb.add_edge(0, 3, 2, 0);
        let g = gb.freeze();

        let diversity = compute_label_diversity(&g, &[0, 1, 2, 3]);
        assert_eq!(3, diversity[0]);
    }

    #[test]
    fn test_in_and_out_counted() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 2, 2, 0);
        gb.add_edge(1, 0, 2, 0);
        let g = gb.freeze();

        let diversity = compute_label_diversity(&g, &[0, 1, 2]);
        assert_eq!(2, diversity[0]);
    }

    #[test]
    fn test_duplicate_neighbors_dont_inflate() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 0);
        gb.add_edge(1, 0, 2, 0);
        let g = gb.freeze();

        let diversity = compute_label_diversity(&g, &[0, 1]);
        assert_eq!(1, diversity[0]);
    }
}
