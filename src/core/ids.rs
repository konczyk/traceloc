use std::collections::HashMap;

pub type NodeId = u32;

pub struct NodeRegistry {
    map: HashMap<String, NodeId>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_or_insert(&mut self, external_id: &str) -> NodeId {
        let next = self.map.len();
        *self.map.entry(external_id.to_string()).or_insert({
            if next == u32::MAX as usize {
                panic!("Nodes count exceeds the limit")
            }
            next as u32
        })
    }

    pub fn get(&self, external_id: &str) -> Option<NodeId> {
        self.map.get(external_id).copied()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
