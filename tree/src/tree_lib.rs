use crate::error::*;

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
pub struct Vertex {
    pub depth: u32,
    pub index: u32,
    pub parent: Option<Arc<Vertex>>,
}

#[derive(Clone, Debug)]
pub struct Tree {
    vertices: HashMap<u32, Arc<Vertex>>,
    deepest: OrdSet<VertexKey>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            vertices: HashMap::new(),
            deepest: OrdSet::new(),
        }
    }

    /// Insert vertex with `event` to the tree
    /// event (uint32 _parent);
    pub fn insert_vertex(&self, event: u32) -> Result<Self> {
        let parent_index = event;

        let mut parent = self.get_vertex_rc(parent_index);
        let index = self.vertices.len() as u32;
        let depth: u32;

        if index == 0 {
            // set parent to none for genesis block
            parent = None;
            depth = 0;
        } else if parent.is_none() {
            return TreeMalformed {
                err: "Incoming vertex doesn't have a valid parent",
            }
            .fail();
        } else {
            depth = parent.clone().unwrap().depth + 1;
        }

        let new_deepest = self.deepest.update(VertexKey { depth, index });

        let vertex: Vertex = Vertex {
            index,
            depth,
            parent,
        };
        let new_vertices = self.vertices.update(index, Arc::new(vertex));

        Ok(Tree {
            deepest: new_deepest,
            vertices: new_vertices,
        })
    }

    /// get ancestor of vertex at depth
    pub fn get_ancestor_rc_at(&self, index: u32, depth: u32) -> Result<Arc<Vertex>> {
        let vertex = self.get_vertex_rc(index);

        if vertex.is_none() {
            // vertex not exist at index
            return VertexNotFound {
                err: "Invalid index",
            }
            .fail();
        }

        let v = vertex.unwrap();
        let vertex_depth = v.depth;

        if vertex_depth == depth {
            // vertex at index is the ancestor at depth itself
            return Ok(v);
        } else if vertex_depth < depth {
            // invalid index or depth
            return VertexNotFound {
                err: "Vertex is not deeper than ancestor",
            }
            .fail();
        } else {
            let mut parent = v.parent.clone();

            // looping through the parent until it reaches `depth` or none
            while parent.is_some() && parent.clone().unwrap().depth > depth {
                parent = parent.unwrap().parent.clone();
            }

            parent
                .filter(|p| p.depth == depth)
                .ok_or(snafu::NoneError)
                .context(TreeMalformed {
                    err: "Ancestor at depth not found",
                })
        }
    }

    /// get index of deepest vertex
    pub fn get_deepest(&self) -> Option<u32> {
        self.deepest.get_max().and_then(|key| Some(key.index))
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: u32) -> Option<&Vertex> {
        self.vertices.get(&index).map(|vertex| Arc::as_ref(vertex))
    }

    /// get vertex by index with reference counter
    pub fn get_vertex_rc(&self, index: u32) -> Option<Arc<Vertex>> {
        self.vertices.get(&index).map(|vertex| Arc::clone(vertex))
    }
}
