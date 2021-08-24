use crate::error::*;

use im::{HashMap, OrdSet};
use snafu::ResultExt;
use std::cmp::Ordering;
use std::sync::Arc;

/// `index` is the unique identifier to each vertex while `depth` is used for
/// sorting. The deepest vertex is defined as largest `depth`, and smallest
/// `index` when the `depth`s are equal, meaning the vertex is oldest in that
/// `depth`
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
    depth: u32,
    index: u32,
    parent: Option<u32>,
}

impl Vertex {
    pub fn get_parent(&self) -> Option<u32> {
        self.parent.clone()
    }

    pub fn get_depth(&self) -> u32 {
        self.depth
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tree {
    vertices: HashMap<u32, Arc<Vertex>>,
    deepest: OrdSet<VertexKey>,
}

impl Tree {
    /// Insert vertex with `event` to the tree
    /// event (uint32 _parent);
    pub fn insert_vertex(&self, event: u32) -> Result<Self> {
        let parent_index = event;

        let mut parent = Some(parent_index);
        let parent_vertex_opt = self.get_vertex_rc(parent_index);
        let index = self.vertices.len() as u32;
        let depth: u32;

        if index == 0 {
            // set parent to none for genesis block
            parent = None;
            depth = 0;
        } else {
            if let Some(parent_vertex) = parent_vertex_opt {
                depth = parent_vertex.depth + 1;
            } else {
                return TreeMalformed {
                    err: "Incoming vertex doesn't have a valid parent",
                }
                .fail();
            }
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
    pub fn get_ancestor_rc_at(
        &self,
        index: u32,
        depth: u32,
    ) -> Result<Arc<Vertex>> {
        let vertex_opt = self.get_vertex_rc(index);

        if let Some(vertex) = vertex_opt {
            let vertex_depth = vertex.depth;

            if vertex_depth == depth {
                // vertex at index is the ancestor at depth itself
                Ok(vertex)
            } else if vertex_depth < depth {
                // invalid index or depth
                VertexNotFound {
                    err: "Vertex is not deeper than ancestor",
                }
                .fail()
            } else {
                let mut parent_opt = vertex.parent.clone();

                // looping through the parent until it reaches `depth` or none
                loop {
                    if let Some(parent) = parent_opt {
                        let parent_vertex = self
                            .get_vertex_rc(parent)
                            .clone()
                            .ok_or(snafu::NoneError)
                            .context(TreeMalformed {
                                err: "Ancestor at depth not found",
                            })?;

                        if parent_vertex.depth <= depth {
                            break;
                        }

                        parent_opt = parent_vertex.parent.clone();
                    } else {
                        break;
                    }
                }

                parent_opt
                    .and_then(|p| self.get_vertex_rc(p))
                    .filter(|p| p.depth == depth)
                    .ok_or(snafu::NoneError)
                    .context(TreeMalformed {
                        err: "Ancestor at depth not found",
                    })
            }
        } else {
            // vertex not exist at index
            return VertexNotFound {
                err: "Invalid index",
            }
            .fail();
        }
    }

    /// get index of last vertex
    pub fn get_last(&self) -> Option<u32> {
        let size = self.size();
        if size == 0 {
            None
        } else {
            Some((size - 1) as u32)
        }
    }

    /// get index of deepest vertex
    pub fn get_deepest(&self) -> Option<u32> {
        self.deepest.get_max().map(|key| key.index)
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: u32) -> Option<&Vertex> {
        self.vertices.get(&index).map(|vertex| Arc::as_ref(vertex))
    }

    /// get vertex by index with reference counter
    pub fn get_vertex_rc(&self, index: u32) -> Option<Arc<Vertex>> {
        self.vertices.get(&index).map(|vertex| Arc::clone(vertex))
    }

    /// is the `vertex` on longest valid path with minimal `distance`
    pub fn is_valid_vertex_with_distance(
        &self,
        index: u32,
        distance: u32,
    ) -> bool {
        if let Some(vertex) = self.get_vertex(index) {
            if let Some(deepest) = self.get_deepest() {
                let deepest_vertex = self.get_vertex(deepest).unwrap();
                let ancestor = self.get_ancestor_rc_at(deepest, vertex.depth);

                if let Ok(ancestor) = ancestor {
                    let ancestor_depth = ancestor.get_depth();
                    let ancestor_index = ancestor.get_index();
                    if (ancestor_index == index)
                        && (deepest_vertex.get_depth() - ancestor_depth
                            >= distance)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// get tree size
    pub fn size(&self) -> usize {
        self.vertices.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::tree_lib::Tree;

    #[test]
    fn test_insert_vertex() {
        let tree = Tree::default().insert_vertex(0);
        assert!(tree.is_ok(), "Insert Genesis Block should pass");

        let tree = tree.unwrap().insert_vertex(5);
        assert!(tree.is_err(), "Insert invalid parent should fail");
    }

    #[test]
    fn test_get_vertex() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }

        for i in 0u32..21 {
            let vertex = tree.get_vertex_rc(i);
            assert!(vertex.is_some(), "Vertex should exist");
            assert!(vertex.unwrap().depth == i, "Vertex depth should match");
        }

        for _ in 0u32..20 {
            tree = tree.insert_vertex(20).unwrap();
        }

        let vertex_20 = tree.get_vertex_rc(20).unwrap();

        for i in 21u32..41 {
            let vertex = tree.get_vertex_rc(i);
            assert!(vertex.is_some(), "Vertex should exist");
            assert!(
                vertex.unwrap().depth == (vertex_20.depth + 1),
                "Vertex depth should match"
            );
        }
    }

    #[test]
    fn test_last() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }

        let last = tree.get_last();
        assert!(last.is_some(), "Last vertex should exist");
        assert!(last.unwrap() == 20, "Last vertex should match");
    }

    #[test]
    fn test_deepest() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }
        for _ in 0u32..5 {
            tree = tree.insert_vertex(20).unwrap();
        }

        let deepest = tree.get_deepest();
        assert!(deepest.is_some(), "Deepest vertex should exist");
        assert!(deepest.unwrap() == 21, "Deepest vertex should match");
    }

    #[test]
    fn test_ancestor() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }

        let last = tree.get_last().unwrap();
        let last_vertex = tree.get_vertex_rc(last).unwrap();

        assert!(
            tree.is_valid_vertex_with_distance(0, 20),
            "Genesis block should be valid and distance 20"
        );
        assert!(
            !tree.is_valid_vertex_with_distance(0, 21),
            "Genesis block should be valid and distance 20"
        );

        assert!(
            tree.is_valid_vertex_with_distance(20, 0),
            "Last block should be valid and distance 0"
        );
        assert!(
            !tree.is_valid_vertex_with_distance(20, 1),
            "Last block should be valid and distance 0"
        );

        for i in 0u32..last_vertex.depth {
            let ancestor = tree.get_ancestor_rc_at(last, i);
            assert!(
                ancestor.is_ok(),
                "Get ancestor on path to Genesis should pass"
            )
        }

        let deepest = tree.get_deepest().unwrap();
        let deepest_vertex = tree.get_vertex_rc(deepest).unwrap();

        for i in 0u32..deepest_vertex.depth {
            let ancestor = tree.get_ancestor_rc_at(deepest, i);
            assert!(
                ancestor.is_ok(),
                "Get ancestor on path to Genesis should pass"
            )
        }
    }
}
