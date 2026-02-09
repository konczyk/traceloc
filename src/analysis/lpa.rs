use crate::core::graph::Graph;
use std::mem::swap;

pub fn label_propagation(graph: &Graph, max_iters: usize) -> Vec<u32> {
    let mut labels = (0..graph.node_count() as u32).collect::<Vec<u32>>();
    let mut next_labels = vec![0; labels.len()];
    for _ in 0..max_iters {
        let mut changed = false;
        let mut seen_labels = Vec::with_capacity(3);
        let mut counts = Vec::with_capacity(3);
        for src in 0..graph.node_count() {
            let mut new_label = labels[src];
            seen_labels.clear();
            counts.clear();
            for n in graph
                .edges_from(src as u32)
                .map(|e| e.dst)
                .chain(graph.edges_to(src as u32).map(|e| e.src))
            {
                let mut seen = false;
                let group = labels[n as usize];
                for i in 0..seen_labels.len() {
                    if seen_labels[i] == group {
                        counts[i] += 1;
                        seen = true;
                        break;
                    }
                }
                if !seen {
                    seen_labels.push(group);
                    counts.push(1);
                }
            }

            if seen_labels.len() > 0 {
                let mut max_idx = 0;
                new_label = seen_labels[max_idx];
                for i in 1..seen_labels.len() {
                    if counts[i] > counts[max_idx]
                        || (counts[i] == counts[max_idx] && seen_labels[i] < seen_labels[max_idx])
                    {
                        max_idx = i;
                        new_label = seen_labels[i];
                    }
                }
            }

            next_labels[src] = new_label;
            if labels[src] != new_label {
                changed = true;
            }
        }
        if changed {
            swap(&mut labels, &mut next_labels);
        } else {
            break;
        }
    }

    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::GraphBuilder;

    #[test]
    fn test_no_edges() {
        let gb = GraphBuilder::new(2);
        let g = gb.freeze();

        let expected = vec![0, 1];
        assert_eq!(expected, label_propagation(&g, 20));
    }

    #[test]
    fn test_simple_chain() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 3);
        gb.add_edge(1, 2, 2, 3);
        gb.add_edge(2, 1, 2, 3);
        let g = gb.freeze();

        let expected = vec![2, 1, 2];
        assert_eq!(expected, label_propagation(&g, 20));
    }

    #[test]
    fn test_dense_groups() {
        let mut gb = GraphBuilder::new(8);
        gb.add_edge(0, 1, 2, 3);
        gb.add_edge(1, 2, 2, 3);
        gb.add_edge(2, 3, 2, 3);
        gb.add_edge(3, 0, 2, 3);
        gb.add_edge(3, 1, 2, 3);
        gb.add_edge(3, 4, 2, 3);
        gb.add_edge(4, 5, 2, 3);
        gb.add_edge(5, 6, 2, 3);
        gb.add_edge(6, 7, 2, 3);
        gb.add_edge(7, 4, 2, 3);
        gb.add_edge(7, 5, 2, 3);
        let g = gb.freeze();

        let expected = vec![1, 0, 1, 0, 3, 4, 3, 4];
        assert_eq!(expected, label_propagation(&g, 3));
    }
}
