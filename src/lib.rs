use std::collections::HashMap;

pub mod dot;
pub mod fileutil;
pub mod graphviz;

#[derive(Debug)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}

pub type NodeId = usize;

pub struct Graph {
    node_arena: Arena,
    edges: Vec<Edge>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            node_arena: Arena::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        let from_id = self.node_arena.insert(from.to_string());
        let to_id = self.node_arena.insert(to.to_string());
        self.edges.push(Edge {
            from: from_id,
            to: to_id,
        });
    }
}

#[derive(Debug)]
struct Arena {
    nodes: Vec<String>,
    inverted_index: HashMap<String, NodeId>,
}

impl Arena {
    fn new() -> Self {
        Arena {
            nodes: Vec::new(),
            inverted_index: HashMap::new(),
        }
    }

    fn insert(&mut self, node: String) -> NodeId {
        *self.inverted_index.entry(node.clone()).or_insert_with(|| {
            let id = self.nodes.len();
            self.nodes.push(node);
            id
        })
    }

    /*
    fn get(&self, id: NodeId) -> Option<&str> {
        self.nodes.get(id).map(|s| s.as_str())
    }
     */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena() {
        let mut arena = Arena::new();
        assert_eq!(arena.insert("a".to_string()), 0);
        assert_eq!(arena.insert("b".to_string()), 1);
        assert_eq!(arena.insert("a".to_string()), 0);
        assert_eq!(arena.insert("c".to_string()), 2);

        assert_eq!(arena.get(0), Some("a"));
        assert_eq!(arena.get(1), Some("b"));
        assert_eq!(arena.get(2), Some("c"));
        assert_eq!(arena.get(3), None);

        assert_eq!(arena.insert("d".to_string()), 3);
        assert_eq!(arena.get(3), Some("d"));
    }
}
