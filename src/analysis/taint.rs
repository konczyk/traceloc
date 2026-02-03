use crate::core::graph::Graph;
use crate::core::ids::NodeId;
use std::collections::{HashMap, VecDeque};

const INITIAL_RISK: f32 = 1.0;
const DECAY: f32 = 0.5;
const EPSILON: f32 = 1e-6;

pub fn propagate(graph: &Graph, start: NodeId, max_hops: usize) -> HashMap<NodeId, f32> {
    let mut risk_map = HashMap::from([(start, INITIAL_RISK)]);
    let mut visited = VecDeque::from([(start, INITIAL_RISK, 0)]);

    while let Some((node, risk, hop)) = visited.pop_front() {
        let new_risk = risk * DECAY;
        if hop == max_hops || new_risk < EPSILON {
            continue;
        }

        let total_amount = graph.edges_from(node).map(|e| e.amount).sum::<u64>();
        if total_amount == 0 {
            continue;
        }
        for edge in graph.edges_from(node) {
            let dilution = edge.amount as f32 / total_amount as f32;
            match risk_map.get(&edge.dst) {
                Some(r) if new_risk <= *r => continue,
                _ => {
                    risk_map.insert(edge.dst, new_risk * dilution);
                }
            }
            visited.push_back((edge.dst, new_risk * dilution, hop + 1))
        }
    }

    risk_map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::GraphBuilder;
    use approx::assert_relative_eq;

    #[test]
    fn test_no_hops() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 3);
        let g = gb.freeze();

        assert_eq!(HashMap::from([(0, 1.0)]), propagate(&g, 0, 0));
    }

    #[test]
    fn test_single_edge() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 1);
        assert_eq!(2, actual.len());
        assert_relative_eq!(1.0f32, actual.get(&0).unwrap());
        assert!(*actual.get(&1).unwrap() < 1.0)
    }

    #[test]
    fn test_hop_limit_enforced() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 2, 3);
        gb.add_edge(1, 2, 2, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 1);
        assert_eq!(2, actual.len());
        assert!(actual.get(&0).is_some());
        assert!(actual.get(&1).is_some());
    }

    #[test]
    fn test_simple_cycle() {
        let mut gb = GraphBuilder::new(2);
        gb.add_edge(0, 1, 2, 3);
        gb.add_edge(1, 0, 2, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 10);
        assert_eq!(2, actual.len());
        assert!(actual.get(&0).is_some());
        assert!(actual.get(&1).is_some());
    }

    #[test]
    fn test_multiple_paths() {
        let mut gb = GraphBuilder::new(4);
        gb.add_edge(0, 1, 2, 3);
        gb.add_edge(1, 3, 2, 3);
        gb.add_edge(0, 2, 2, 3);
        gb.add_edge(2, 3, 2, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 10);
        assert_eq!(4, actual.len());
        assert!(actual.get(&0).is_some());
        assert!(actual.get(&1).is_some());
        assert!(actual.get(&2).is_some());
        assert!(actual.get(&3).is_some());
    }

    #[test]
    fn test_fan_out_dilution() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 100, 3);
        gb.add_edge(0, 2, 1, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 1);
        assert_eq!(3, actual.len());
        assert!(actual.get(&1).unwrap() > actual.get(&2).unwrap());
    }

    #[test]
    fn test_zero_amounts() {
        let mut gb = GraphBuilder::new(3);
        gb.add_edge(0, 1, 0, 3);
        gb.add_edge(0, 2, 0, 3);
        let g = gb.freeze();

        let actual = propagate(&g, 0, 1);
        assert_eq!(1, actual.len());
        assert!(actual.get(&0).is_some());
    }
}
