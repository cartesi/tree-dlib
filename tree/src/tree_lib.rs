use state_actor::error::*;

use im::{OrdSet, Vector};
use std::cmp::Ordering;

/// `index` is the unique identifier to each vertex while `depth` is used for sorting.
/// The deepest vertex is defined as largest `depth`, and smallest `index` when the `depth`s are equal,
/// meaning the vertex is oldest in that `depth`
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq)]
struct VertexKey {
    depth: u32,
    index: u32,
}

impl Ord for VertexKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let depth_cmp = self.depth.cmp(&other.depth);
        match depth_cmp {
            Ordering::Equal => other.index.cmp(&self.index),
            _ => depth_cmp,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vertex<T>
where
    T: Clone,
{
    pub ancestors: Vec<u32>,
    pub children: Vec<u32>,
    pub data: T,
    pub depth: u32,
    pub has_pruned: bool,
}

#[derive(Clone, Debug)]
pub struct Tree<T>
where
    T: Clone,
{
    vertices: Vector<Vertex<T>>,
    deepest: OrdSet<VertexKey>,
}

impl<T> Tree<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Tree {
            vertices: Vector::new(),
            deepest: OrdSet::new(),
        }
    }

    /// Adds vertices with index `new_vertices` to the tree
    pub async fn add_vertices(&mut self, new_vertices: Vec<(u32, Vertex<T>)>) -> Result<()> {
        for (index, mut vertex) in new_vertices {
            if index as usize != self.vertices.len() {
                return BlockchainInconsistent {
                    err: "Inconsistent tree size",
                }
                .fail();
            }
            if let Some(parent_index) = vertex.ancestors.get(0) {
                if let Some(mut parent) = self.get_vertex(*parent_index) {
                    if parent.has_pruned {
                        // prune vertex if parent is pruned
                        vertex.has_pruned = true;
                    }
                    // update children list of parent
                    parent.children.push(index);
                    self.vertices.set(*parent_index as usize, parent);
                }
            }

            self.deepest.insert(VertexKey {
                depth: vertex.depth,
                index,
            });
            self.vertices.push_back(vertex);
        }

        Ok(())
    }

    /// get ancestor of vertex at depth
    pub fn get_ancestor_at(&self, index: u32, depth: u32) -> Option<Vertex<T>> {
        // TODO: should consider has_pruned?
        match self.get_vertex(index) {
            Some(v) => {
                if v.depth < depth {
                    // vertex must be deeper that ancestor
                    None
                } else if v.depth == depth {
                    // ancestor found
                    Some(v)
                } else {
                    // start jumping from oldest ancestor
                    for ancestor in v.ancestors.into_iter().rev() {
                        if let Some(anc) = self.get_vertex(ancestor) {
                            // closest ancestor found, recursive the search from there
                            if anc.depth >= depth {
                                return self.get_ancestor_at(ancestor, depth);
                            }
                        }
                    }
                    // no ancestor at depth
                    // TODO: should be an error? panic?
                    None
                }
            }
            // vertex not found with given index
            None => None,
        }
    }

    /// get deepest vertex
    pub fn get_deepest(&self) -> Option<(u32, Vertex<T>)> {
        match self.deepest.get_max() {
            Some(k) => self
                .vertices
                .get(k.index as usize)
                .map(|vertex| (k.index, vertex.clone())),
            None => None,
        }
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: u32) -> Option<Vertex<T>> {
        self.vertices
            .get(index as usize)
            .map(|vertex| vertex.clone())
    }

    pub fn prune_vertex(&mut self, index: u32) {
        if let Some(vertex) = self.vertices.get(index as usize) {
            if !vertex.has_pruned {
                let mut v = vertex.clone();
                let children = vertex.children.clone();

                // prune children recursively
                for child in children {
                    self.prune_vertex(child);
                }

                // prune vertex
                v.has_pruned = true;
                self.vertices.set(index as usize, v);
            }
        }
    }
}
