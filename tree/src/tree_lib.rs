use state_actor::error::*;

use im::{HashMap, OrdSet};
use snafu::ResultExt;
use std::cmp::Ordering;
use std::sync::Arc;

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
    // pub ancestors: Vec<u32>,
    // pub children: Vec<u32>,
    pub data: T,
    pub depth: u32,
    pub parent: Option<u32>,
    // pub has_pruned: bool,
}

#[derive(Clone, Debug)]
pub struct Tree<T>
where
    T: Clone,
{
    vertices: HashMap<u32, Arc<Vertex<T>>>,
    deepest: OrdSet<VertexKey>,
}

impl<T> Tree<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Tree {
            vertices: HashMap::new(),
            deepest: OrdSet::new(),
        }
    }

    /// Adds vertex with `new_vertex` to the tree
    pub fn add_vertex(&self, new_vertex: (u32, Vertex<T>)) -> Result<Self> {
        let (index, vertex) = (new_vertex.0, new_vertex.1);

        if index as usize != self.vertices.len() {
            return BlockchainInconsistent {
                err: "Inconsistent tree size",
            }
            .fail();
        }

        let new_deepest = self.deepest.update(VertexKey {
            depth: vertex.depth,
            index,
        });

        let new_vertices = self.vertices.update(index, Arc::new(vertex));

        Ok(Tree {
            deepest: new_deepest,
            vertices: new_vertices,
        })
    }

    /// get index of ancestor of vertex at depth
    pub fn get_ancestor_at(&self, index: u32, depth: u32) -> Result<u32> {
        self.get_vertex(index)
            .and_then(|v| {
                if v.depth == depth {
                    Some(index)
                } else if v.depth < depth {
                    None
                } else {
                    v.parent
                        .and_then(|parent| self.get_ancestor_at(parent, depth).ok())
                }
            })
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "No ancestor at target depth",
            })
    }

    /// get index of deepest vertex
    pub fn get_deepest(&self) -> Option<u32> {
        self.deepest.get_max().and_then(|key| Some(key.index))
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: u32) -> Option<&Vertex<T>> {
        self.vertices.get(&index).map(|vertex| Arc::as_ref(vertex))
    }

    /// get vertex by index with reference counter
    pub fn get_vertex_rc(&self, index: u32) -> Option<Arc<Vertex<T>>> {
        self.vertices.get(&index).map(|vertex| Arc::clone(vertex))
    }
}
