use std::collections::{HashMap, HashSet};

use crate::mermaid_flow::{
    FlowChartMulitpleLinks, FlowChartPairNodes, FlowchartMultipleNodesLinks, Stmt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone)]
pub struct Edge {
    pub to: NodeId,
    pub data : Vec<String>,
}

impl Edge {
    pub fn new2(to: NodeId, data: Vec<String>) -> Self {
        Self { to, data }
    }

    pub fn new(to:NodeId) -> Self {
        Self { to, data: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Graph<N> {
    nodes: Vec<N>,
    adj: HashMap<NodeId, Vec<Edge>>,
}

impl<N> Graph<N> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            adj: HashMap::new(),
        }
    }

    pub fn get_nodes(&self) -> &Vec<N> {
        &self.nodes
    }

    pub fn add_node(&mut self, data: N) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(data);
        self.adj.insert(id, Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.adj
            .get_mut(&from)
            .expect("invalid from node")
            .push(Edge::new(to));
    }

    pub fn node(&self, id: NodeId) -> &N {
        &self.nodes[id.0]
    }

    pub fn outgoing_neighbors(&self, id: NodeId) -> impl Iterator<Item = &Edge> {
        self.adj.get(&id).into_iter().flatten()
    }

    pub fn neighbors(&self, id: NodeId) -> impl Iterator<Item = Edge> + '_ {
        let outgoing = self
            .outgoing_neighbors(id)
            .cloned()
            .collect::<Vec<_>>();
        let incoming = self
            .adj
            .iter()
            .flat_map(move |(from, edges)| {
                edges.iter().filter_map(move |edge| {
                    if edge.to == id {
                        Some(Edge {
                            to: *from,
                            data: edge.data.clone(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        outgoing.into_iter().chain(incoming)
    }

    pub fn neighbor_edges(&self, id: NodeId) -> Vec<(NodeId, NodeId, Edge)> {
        let outgoing = self
            .outgoing_neighbors(id)
            .cloned()
            .map(|edge| (id, edge.to, edge))
            .collect::<Vec<_>>();
        let incoming = self
            .adj
            .iter()
            .flat_map(|(from, edges)| {
                edges.iter().filter_map(|edge| {
                    if edge.to == id {
                        Some((*from, id, edge.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        outgoing.into_iter().chain(incoming).collect()
    }

    pub fn neighbor_nodes(&self, id: NodeId) -> Vec<NodeId> {
        let mut seen = HashSet::new();
        let mut nodes = Vec::new();

        for edge in self.outgoing_neighbors(id) {
            if seen.insert(edge.to) {
                nodes.push(edge.to);
            }
        }

        for (from, edges) in &self.adj {
            for edge in edges {
                if edge.to == id && seen.insert(*from) {
                    nodes.push(*from);
                }
            }
        }

        nodes
    }

    pub fn edge_between(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
        self.adj
            .get(&from)
            .and_then(|edges| edges.iter().find(|edge| edge.to == to))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_node_id_by_name(&self, name: &str) -> Option<NodeId>
        where
            N: AsRef<str>,
    {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.as_ref() == name)
            .map(|(i, _)| NodeId(i))
    }

    pub fn is_tree(&self) -> bool {
        let n = self.node_count();
        if n == 0 {
            return false;
        }

        let mut indegree = vec![0usize; n];
        for edges in self.adj.values() {
            for e in edges {
                indegree[e.to.0] += 1;
            }
        }

        let roots: Vec<NodeId> = indegree
            .iter()
            .enumerate()
            .filter(|&(_, &d)| d == 0)
            .map(|(i, _)| NodeId(i))
            .collect();

        if roots.len() != 1 {
            return false;
        }

        let root = roots[0];
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        fn dfs<N>(
            g: &Graph<N>,
            u: NodeId,
            visited: &mut HashSet<NodeId>,
            visiting: &mut HashSet<NodeId>,
        ) -> bool {
            if visiting.contains(&u) {
                return false;
            }
            if visited.contains(&u) {
                return true;
            }

            visiting.insert(u);
            for e in g.outgoing_neighbors(u) {
                if !dfs(g, e.to, visited, visiting) {
                    return false;
                }
            }
            visiting.remove(&u);
            visited.insert(u);
            true
        }

        if !dfs(self, root, &mut visited, &mut visiting) {
            return false;
        }

        visited.len() == n
    }

    pub fn tree_root(&self) -> Result<NodeId, TreeError> {
        let n = self.node_count();
        if n == 0 {
            return Err(TreeError::Empty);
        }

        let mut indegree = vec![0usize; n];
        for edges in self.adj.values() {
            for e in edges {
                indegree[e.to.0] += 1;
            }
        }

        let roots: Vec<NodeId> = indegree
            .iter()
            .enumerate()
            .filter(|(_, d)| **d == 0)
            .map(|(i, _)| NodeId(i))
            .collect();

        if roots.is_empty() {
            return Err(TreeError::NoRoot);
        }
        if roots.len() > 1 {
            return Err(TreeError::MultipleRoots);
        }

        let root = roots[0];
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        fn dfs<N>(
            g: &Graph<N>,
            u: NodeId,
            visited: &mut HashSet<NodeId>,
            visiting: &mut HashSet<NodeId>,
        ) -> Result<(), TreeError> {
            if visiting.contains(&u) {
                return Err(TreeError::Cycle);
            }
            if visited.contains(&u) {
                return Ok(());
            }

            visiting.insert(u);
            for e in g.outgoing_neighbors(u) {
                dfs(g, e.to, visited, visiting)?;
            }
            visiting.remove(&u);
            visited.insert(u);
            Ok(())
        }

        dfs(self, root, &mut visited, &mut visiting)?;
        if visited.len() != n {
            return Err(TreeError::Disconnected);
        }

        Ok(root)
    }

    pub fn tree_depth(&self, root: NodeId) -> usize {
        fn dfs<N>(g: &Graph<N>, u: NodeId) -> usize {
            let mut max_child_depth = 0;
            for e in g.outgoing_neighbors(u) {
                let d = dfs(g, e.to);
                max_child_depth = max_child_depth.max(d);
            }
            1 + max_child_depth
        }

        dfs(self, root)
    }
}

#[derive(Debug)]
pub enum TreeError {
    Empty,
    NoRoot,
    MultipleRoots,
    Cycle,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct FCNode {
    pub name: String,
    pub data: String,
}

impl AsRef<str> for FCNode {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

pub struct FlowchartToGraph {
    node_map: HashMap<String, NodeId>,
}

impl FlowchartToGraph {
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
        }
    }

    fn intern_node(
        &mut self,
        g: &mut Graph<FCNode>,
        name: String,
        data: String,
    ) -> NodeId {
        if let Some(&id) = self.node_map.get(&name) {
            return id;
        }

        let id = g.add_node(FCNode {
            name: name.clone(),
            data,
        });
        self.node_map.insert(name, id);
        id
    }

    pub fn convert(mut self, stmts: &[Stmt]) -> Graph<FCNode> {
        let mut graph = Graph::new();

        for stmt in stmts {
            self.convert_stmt(&mut graph, stmt);
        }

        graph
    }

    fn convert_stmt(&mut self, g: &mut Graph<FCNode>, stmt: &Stmt) {
        match stmt {
            Stmt::FlowChartMulitpleLinks(m) => {
                self.convert_multiple_links(g, m);
            }
            Stmt::FlowchartMultipleNodesLinks(m) => {
                self.convert_multiple_nodes_links(g, m);
            }
            Stmt::FlowChartGraph(sub) => {
                for s in sub.get_stmts() {
                    self.convert_stmt(g, s);
                }
            }
        }
    }

    fn convert_multiple_links(
        &mut self,
        g: &mut Graph<FCNode>,
        m: &FlowChartMulitpleLinks,
    ) {
        let pairs: &FlowChartPairNodes = m.get_flowchart_mutiple_links();

        for (from, link, to) in pairs.get_flowchart_pair_nodes() {
            let from_id = self.intern_node(
                g,
                from.get_node_name().to_owned(),
                from.get_node_text().clone(),
            );
            let to = match to {
                Some(n) => n,
                None => continue,
            };
            let to_id = self.intern_node(g, to.get_node_name().to_owned(), to.get_node_text().clone());

            let _label = link.as_ref().map(|l| l.as_str().to_owned());
            g.add_edge(from_id, to_id);
        }
    }

    fn convert_multiple_nodes_links(
        &mut self,
        _g: &mut Graph<FCNode>,
        _m: &FlowchartMultipleNodesLinks,
    ) {
        todo!("not implemented yet for FlowchartMultipleNodesLinks");
    }
}

pub trait TreeView {
    type NodeId: Copy + Eq;

    fn children(&self, node: Self::NodeId) -> Vec<Self::NodeId>;
    fn node_name(&self, node: Self::NodeId) -> &str;
}

pub struct GraphTreeView<'a, N> {
    graph: &'a Graph<N>,
}

impl<'a, N> GraphTreeView<'a, N> {
    pub fn new(graph: &'a Graph<N>, _root: NodeId) -> Self {
        Self { graph }
    }
}

impl<'a, N> TreeView for GraphTreeView<'a, N>
where
    N: AsRef<str>,
{
    type NodeId = NodeId;

    fn children(&self, node: Self::NodeId) -> Vec<Self::NodeId> {
        self.graph.outgoing_neighbors(node).map(|e| e.to).collect()
    }

    fn node_name(&self, node: Self::NodeId) -> &str {
        self.graph.node(node).as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbors_include_incoming_and_outgoing_edges() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let neighbor_ids = graph
            .neighbors(b)
            .map(|edge| edge.to)
            .collect::<HashSet<_>>();

        assert_eq!(neighbor_ids, HashSet::from([a, c]));
    }

    #[test]
    fn outgoing_neighbors_remain_directed() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b);

        let neighbor_ids = graph
            .outgoing_neighbors(b)
            .map(|edge| edge.to)
            .collect::<Vec<_>>();

        assert!(neighbor_ids.is_empty());
    }

    #[test]
    fn neighbor_edges_include_from_to_and_edge() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let neighbor_edges = graph.neighbor_edges(b);

        assert!(neighbor_edges
            .iter()
            .any(|(from, to, edge)| *from == a && *to == b && edge.to == b));
        assert!(neighbor_edges
            .iter()
            .any(|(from, to, edge)| *from == b && *to == c && edge.to == c));
    }

    #[test]
    fn neighbor_nodes_include_all_connected_nodes_once() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b);
        graph.add_edge(a, b);
        graph.add_edge(b, c);
        graph.add_edge(c, b);

        let neighbors = graph.neighbor_nodes(b).into_iter().collect::<HashSet<_>>();

        assert_eq!(neighbors, HashSet::from([a, c]));
    }

    #[test]
    fn edge_between_returns_edge_when_exists() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b);

        let edge = graph.edge_between(a, b);
        assert!(edge.is_some());
        assert_eq!(edge.unwrap().to, b);
        assert!(graph.edge_between(b, a).is_none());
    }
}