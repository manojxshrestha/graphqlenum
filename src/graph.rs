use crate::introspection;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    mutation_node_index: Option<usize>,
}

pub type NodeIndex = usize;

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub edges: Vec<EdgeIndex>,
}

pub type EdgeIndex = usize;
pub type ResultPair = (NodeIndex, Vec<EdgeIndex>);

#[derive(Debug)]
pub struct Edge {
    pub name: String,
    pub destination: NodeIndex,
}

enum NodeMapItem<'a> {
    NewNode(&'a introspection::SchemaType),
    CachedNode(NodeIndex),
}

impl Graph {
    pub fn new(
        schema: introspection::Schema,
        show_connections: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let mut graph = Self {
            nodes: vec![],
            edges: vec![],
            mutation_node_index: None,
        };

        let mut type_map: HashMap<String, NodeMapItem> = schema
            .types
            .iter()
            .map(|x| (x.name.clone(), NodeMapItem::NewNode(x)))
            .collect();

        let query_node = match type_map.remove(&schema.query_type.name) {
            Some(NodeMapItem::NewNode(node)) => node,
            _ => return Err(From::from("Input data contains no query node.")),
        };

        graph.add_node(query_node, &mut type_map, show_connections)?;

        if let Some(mutation_type) = &schema.mutation_type {
            let mutation_node = match type_map.remove(&mutation_type.name) {
                Some(NodeMapItem::NewNode(node)) => node,
                _ => return Err(From::from("Input data contains no mutation node.")),
            };

            graph.mutation_node_index = Some(graph.nodes.len());
            graph.add_node(mutation_node, &mut type_map, show_connections)?;
        }

        Ok(graph)
    }

    fn add_node(
        &mut self,
        schema_type: &introspection::SchemaType,
        type_map: &mut HashMap<String, NodeMapItem>,
        show_connections: bool,
    ) -> Result<NodeIndex, Box<dyn Error>> {
        let node = Node::new(schema_type);
        let index = self.nodes.len();
        type_map.insert(node.name.clone(), NodeMapItem::CachedNode(index));
        self.nodes.push(node);

        if let Some(fields) = &schema_type.fields {
            for field in fields {
                let field_name = if show_connections {
                    field.field_type.get_graph_object_name()
                } else {
                    match self.get_connection_object(field, type_map)? {
                        None => field.field_type.get_graph_object_name(),
                        connection_object => connection_object,
                    }
                };

                let destination = match field_name {
                    Some(name) => match type_map.remove(&name) {
                        Some(NodeMapItem::NewNode(new_node)) => {
                            Some(self.add_node(new_node, type_map, show_connections)?)
                        }
                        Some(NodeMapItem::CachedNode(node_index)) => {
                            // Cached nodes have to be put back in
                            type_map.insert(name.to_string(), NodeMapItem::CachedNode(node_index));
                            Some(node_index)
                        }
                        None => {
                            return Err(From::from(format!(
                                "Node \"{}\" was not found in the type map.",
                                name
                            )))
                        }
                    },
                    _ => None, // The field has a non-object type
                };

                if let Some(destination) = destination {
                    let edge = self.add_edge(field, destination);
                    // I can't use the `node` variable because it has been pushed into the `self.nodes` vector
                    // and this method doesn't own it anymore. I feel like looking it up again in the vector
                    // probably isn't the best way to do this but that's all I could come up with for now.
                    self.nodes[index].edges.push(edge);
                }
            }
        }

        Ok(index)
    }

    fn add_edge(&mut self, field: &introspection::Field, destination: NodeIndex) -> EdgeIndex {
        let index = self.edges.len();
        self.edges.push(Edge::new(field, destination));

        index
    }

    pub fn enumerate_paths_to_target(
        &self,
        destination: &str,
        include_mutations: bool,
    ) -> Result<Vec<ResultPair>, Box<dyn Error>> {
        let destination_index = match self.nodes.iter().position(|x| x.name == destination) {
            Some(x) => x,
            None => {
                return Err(From::from(format!(
                    "Could not find a node named \"{}\" in the graph.",
                    destination
                )))
            }
        };

        let result: Vec<ResultPair> = vec![];
        let mut result = self.enumerate_paths_to_target_from_index(0, destination_index, result)?;

        if include_mutations {
            if let Some(mutation_node_index) = self.mutation_node_index {
                result = self.enumerate_paths_to_target_from_index(
                    mutation_node_index,
                    destination_index,
                    result,
                )?;
            }
        }

        Ok(result)
    }

    fn enumerate_paths_to_target_from_index(
        &self,
        origin_index: usize,
        destination_index: usize,
        mut result: Vec<ResultPair>,
    ) -> Result<Vec<ResultPair>, Box<dyn Error>> {
        let starting_node = match self.nodes.get(origin_index) {
            Some(x) => x,
            None => return Err(From::from("Could not find the root node.")),
        };

        for root_node_edge_index in &starting_node.edges {
            let mut stack: Vec<EdgeIndex> = vec![*root_node_edge_index];
            let mut visited: Vec<EdgeIndex> = vec![];
            self.dfs(
                origin_index,
                self.edges[*root_node_edge_index].destination,
                destination_index,
                &mut stack,
                &mut result,
                &mut visited,
            );
            assert_eq!(stack.len(), 1);
        }

        Ok(result)
    }

    fn dfs(
        &self,
        origin: NodeIndex,
        start: NodeIndex,
        destination: NodeIndex,
        stack: &mut Vec<EdgeIndex>,
        result: &mut Vec<ResultPair>,
        visited: &mut Vec<EdgeIndex>,
    ) {
        if destination == start {
            result.push((origin, stack.clone()));
        } else {
            let node = &self.nodes[start];
            for edge in &node.edges {
                if visited.contains(edge) {
                    return;
                } else {
                    visited.push(*edge);
                    stack.push(*edge);
                    self.dfs(
                        origin,
                        self.edges[*edge].destination,
                        destination,
                        stack,
                        result,
                        visited,
                    );
                    assert!(stack.pop().is_some());
                }
            }
        }
    }

    fn get_connection_object(
        &self,
        field: &introspection::Field,
        type_map: &mut HashMap<String, NodeMapItem>,
    ) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(connection_type_name) = field.get_connection_type_name() {
            return match type_map.get(&connection_type_name) {
                Some(NodeMapItem::NewNode(node)) => {
                    if let Some(fields) = &node.fields {
                        for field in fields {
                            if let Some(field_type_name) = field.field_type.get_graph_object_name()
                            {
                                // We want what is usually named the "nodes" field, but it isn't always named like that...
                                if field_type_name != "pageInfo"
                                    && !field_type_name.ends_with("Edge")
                                {
                                    return Ok(Some(field_type_name));
                                }
                            }
                        }
                    }
                    Err(From::from(format!(
                        "Cannot find connection type for \"{}\".",
                        connection_type_name
                    )))
                }
                Some(NodeMapItem::CachedNode(_)) => {
                    // We should only find new nodes
                    Err(From::from("Node cache appears to be corrupted."))
                }
                None => Err(From::from(format!(
                    "Node \"{}\" was not found in the type map.",
                    connection_type_name
                ))),
            };
        }

        Ok(None)
    }
}

impl Node {
    fn new(schema_type: &introspection::SchemaType) -> Node {
        Node {
            name: schema_type.name.clone(),
            edges: vec![],
        }
    }
}

impl Edge {
    fn new(field: &introspection::Field, destination: NodeIndex) -> Edge {
        Edge {
            name: field.name.clone(),
            destination,
        }
    }
}
