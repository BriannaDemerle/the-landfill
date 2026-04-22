//! I just looked at the wikipedia pseudocode...
//! `Coded on 4/22/26`
//! ```rust
//! let nodes: [char; 11] =
//! std::array::from_fn(|index| ('a'..='z').nth(index).unwrap());
//! let mut g = Graph::with_nodes(nodes);
//!
//! //     b---9---e            i
//! //    /|\     /|\          /
//! //   2 | 8   4 1 2        4
//! //  /  |  \ /  |  \      /
//! // a   2   d-3-f-1-h    j
//! //  \  |  / \  |  /
//! //   5 | 2   1 1 3
//! //    \|/     \|/
//! //     c---4---g
//!
//! g.set_edge_symmetric(&'a', &'b', 2).unwrap();
//! g.set_edge_symmetric(&'a', &'c', 5).unwrap();
//! g.set_edge_symmetric(&'b', &'c', 2).unwrap();
//! g.set_edge_symmetric(&'b', &'d', 8).unwrap();
//! g.set_edge_symmetric(&'b', &'e', 9).unwrap();
//! g.set_edge_symmetric(&'c', &'d', 2).unwrap();
//! g.set_edge_symmetric(&'c', &'g', 4).unwrap();
//! g.set_edge_symmetric(&'d', &'e', 4).unwrap();
//! g.set_edge_symmetric(&'d', &'f', 3).unwrap();
//! g.set_edge_symmetric(&'d', &'g', 1).unwrap();
//! g.set_edge_symmetric(&'e', &'f', 1).unwrap();
//! g.set_edge_symmetric(&'e', &'h', 2).unwrap();
//! g.set_edge_symmetric(&'f', &'g', 1).unwrap();
//! g.set_edge_symmetric(&'f', &'h', 1).unwrap();
//! g.set_edge_symmetric(&'g', &'h', 3).unwrap();
//! g.set_edge_symmetric(&'i', &'j', 4).unwrap();
//!
//! assert_eq!(
//!     g.shortest_path(&'a', &'h').unwrap(),
//!     Some(PathData {
//!         path: vec![&'a', &'b', &'c', &'d', &'g', &'f', &'h'],
//!         distance: 10
//!     })
//! );
//!
//! assert_eq!(
//!     g.shortest_path(&'a', &'a').unwrap(),
//!     Some(PathData {
//!         path: vec![&'a'],
//!         distance: 0
//!     })
//! );
//!
//! assert_eq!(
//!     g.shortest_path(&'j', &'i').unwrap(),
//!     Some(PathData {
//!         path: vec![&'j', &'i'],
//!         distance: 4
//!     })
//! );
//!
//! assert!(g.shortest_path(&'a', &'j').unwrap().is_none());
//! ```

/// A collection of useful structs for performing Dijkstra's algorithm.
mod dijkstra {
    /// A struct representing a path on a graph of `Node`s.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PathData<Node> {
        pub path: Vec<Node>,
        pub distance: usize,
    }

    /// Holds the predecessor of this node in the shortest path to a
    /// starting node, and the distance to that node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DistanceData {
        pub predecessor: usize,
        pub distance: usize,
    }

    /// The data about node when performing Dijkstra's algorithm.
    /// The data includes the optional `DistanceData`, and whether or not
    /// the current node has its distance finalized.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NodeData {
        pub distance_data: Option<DistanceData>,
        pub is_finalized: bool,
    }

    impl NodeData {
        /// Gets the distance from the start node, if one is present.
        pub fn distance(&self) -> Option<usize> {
            self.distance_data
                .map(|DistanceData { distance, .. }| distance)
        }

        /// Gets the previous node from this one in the shortest path
        /// found so far, if a path exists.
        pub fn predecessor(&self) -> Option<usize> {
            self.distance_data
                .map(|DistanceData { predecessor, .. }| predecessor)
        }
    }

    impl PartialOrd for NodeData {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for NodeData {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            Option::<usize>::cmp(&self.distance(), &other.distance())
        }
    }
}

use dijkstra::{DistanceData, NodeData};
pub use dijkstra::PathData;

/// A graph of `N` `Node`s represented with an adjacency matrix.
#[derive(Debug, Clone)]
pub struct Graph<Node, const N: usize> {
    adjacencies: [[usize; N]; N],
    nodes: [Node; N],
}

/// The types of errors a graph can have.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GraphError {
    FromNodeNotFound,
    DestNodeNotFound,
}

impl<Node: Clone + Eq, const N: usize> Graph<Node, N> {
    /// Creates a graph with the given nodes and no edges.
    pub fn with_nodes(nodes: [Node; N]) -> Self {
        Self {
            adjacencies: [[0; N]; N],
            nodes,
        }
    }

    /// Gets the internal index of the node in the graph if present.
    /// This index is the index used for the adjacency matrix.
    fn get_node_index(&self, node: &Node) -> Option<usize> {
        self.nodes.iter().position(|other| node == other)
    }

    /// Gets the node from its index if the index is in bounds (`index < N`).
    fn get_node(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    /// Gets the edge between `from` and `dest` if both are indices
    /// within bounds. `Some(0)` is returned if the nodes do not have an edge
    /// but are both valid node indices.
    fn get_edge(&self, from: usize, dest: usize) -> Option<usize> {
        self.adjacencies.get(from)?.get(dest).copied()
    }

    /// Gets a mutable reference to the edge between `from` and `dest`
    /// if present.
    fn get_edge_mut(&mut self, from: usize, dest: usize) -> Option<&mut usize> {
        self.adjacencies.get_mut(from)?.get_mut(dest)
    }

    /// Sets the edge between `from` and `dest` with the specified weight,
    /// returning if there was any accessing error.
    pub fn set_edge(
        &mut self,
        from: &Node,
        dest: &Node,
        weight: usize,
    ) -> Result<(), GraphError> {
        let from = self
            .get_node_index(from)
            .ok_or(GraphError::FromNodeNotFound)?;
        let dest = self
            .get_node_index(dest)
            .ok_or(GraphError::DestNodeNotFound)?;

        *self.get_edge_mut(from, dest).unwrap() = weight;

        Ok(())
    }

    /// Sets the edges between `from` and `dest`, and `dest` and `from`
    /// with the specified weight, returning if there was any accessing error.
    pub fn set_edge_symmetric(
        &mut self,
        from: &Node,
        dest: &Node,
        weight: usize,
    ) -> Result<(), GraphError> {
        self.set_edge(from, dest, weight)?;
        self.set_edge(dest, from, weight)?;
        Ok(())
    }

    /// Returns an iterator of all the neighbor-weight pairs.
    fn neighbors(&self, node: usize) -> impl Iterator<Item = (usize, usize)> {
        (0..N)
            .map(move |n| (n, self.get_edge(node, n)))
            .filter_map(|(n, edge)| edge.map(|e| (n, e)))
            .filter(|&(_, edge)| edge != 0)
    }

    /// Builds a path of nodes from `from` to `dest`
    /// using the completed node data from Dijkstra's algorithm.
    fn build_path(
        &self,
        from: usize,
        dest: usize,
        node_data: &[NodeData],
    ) -> Option<PathData<&Node>> {
        if let Some(distance) = node_data[dest].distance() {
            let mut current_node = dest;
            let mut path = vec![dest];

            while let Some(predecessor) = node_data[current_node].predecessor()
            {
                if current_node == from {
                    break;
                }

                path.push(predecessor);

                current_node = predecessor;
            }

            Some(PathData {
                path: path
                    .iter()
                    .rev()
                    .map(|&index| self.get_node(index).unwrap())
                    .collect(),
                distance,
            })
        } else {
            None
        }
    }

    /// Finds the shortest path from `from` to `dest` if both nodes exist
    /// using Dijkstra's algorithm.
    pub fn shortest_path(
        &self,
        from: &Node,
        dest: &Node,
    ) -> Result<Option<PathData<&Node>>, GraphError> {
        let from = self
            .get_node_index(from)
            .ok_or(GraphError::FromNodeNotFound)?;
        let dest = self
            .get_node_index(dest)
            .ok_or(GraphError::DestNodeNotFound)?;

        let mut node_data = [NodeData::default(); N];

        node_data[from] = NodeData {
            distance_data: Some(DistanceData {
                predecessor: from,
                distance: 0,
            }),
            is_finalized: false,
        };

        while let Some((node_index, current_data)) = node_data
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, visiting)| !visiting.is_finalized)
            .filter(|(_, visiting)| visiting.distance_data.is_some())
            .min()
        {
            node_data[node_index].is_finalized = true;

            for (neighbor, weight) in self.neighbors(node_index) {
                let new_distance = current_data.distance().unwrap() + weight;
                let old_distance = node_data[neighbor].distance();

                if old_distance.is_none() || Some(new_distance) < old_distance {
                    node_data[neighbor].distance_data = Some(DistanceData {
                        predecessor: node_index,
                        distance: new_distance,
                    });
                }
            }
        }

        Ok(self.build_path(from, dest, &node_data))
    }
}