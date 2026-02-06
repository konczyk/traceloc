use crate::core::graph::{Graph, GraphBuilder};
use std::collections::HashMap;
use std::mem::swap;

pub fn label_propagation(graph: &Graph, max_iters: usize) -> Vec<u32> {
    let mut labels = (0..graph.node_count() as u32).collect::<Vec<u32>>();
    let mut next_labels = vec![0; labels.len()];
    for _ in 0..max_iters {
        let mut changed = false;
        for src in 0..graph.node_count() {
            let mut new_label = labels[src];
            let mut counts = HashMap::new();

            for n in graph
                .edges_from(src as u32)
                .map(|e| e.dst)
                .chain(graph.edges_to(src as u32))
            {
                let group = labels[n as usize];
                counts.entry(group).and_modify(|cnt| *cnt += 1).or_insert(1);
                let group_cnt = *counts.get(&group).unwrap_or(&0);
                let prev_cnt = *counts.get(&new_label).unwrap_or(&0);

                if group_cnt > prev_cnt || (group_cnt == prev_cnt && group < new_label) {
                    new_label = group;
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
