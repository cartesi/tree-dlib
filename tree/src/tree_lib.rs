use state_actor::error::*;

use futures::future::join_all;
use im::{HashMap, OrdSet};
use std::cmp::Ordering;
use std::future::Future;

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

#[derive(Clone, Debug)]
pub struct Vertex<T>
where
    T: Clone,
{
    ancestors: Vec<u32>,
    children: Vec<u32>,
    data: T,
    depth: u32,
    has_pruned: bool,
}

#[derive(Clone, Debug)]
pub struct Tree<T>
where
    T: Clone,
{
    vertices: HashMap<u32, Vertex<T>>,
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

    /// Adds vertices with index `new_vertices` to the tree, calling
    /// `onchain_get_vertex` for each index.
    pub async fn add_vertices<F>(
        &mut self,
        new_indices: &Vec<u32>,
        onchain_get_vertex: impl Fn(u32) -> F,
    ) -> Result<()>
    where
        // (T, depth, ancestors)
        F: Future<Output = Result<(T, u32, Vec<u32>)>>,
    {
        let mut futures = vec![];

        for new_index in new_indices {
            let future = {
                let idx = new_index.clone();
                let vertex = onchain_get_vertex(idx.clone());
                async move { (idx, vertex.await) }
            };
            futures.push(future);
        }

        let vertices = join_all(futures).await;

        for (index, vertex) in vertices {
            let v = vertex?;

            if let Some(parent_index) = v.2.get(0) {
                if let Some(mut parent) = self.get_vertex(*parent_index) {
                    if parent.has_pruned {
                        // ignore adding vertex to pruned parent
                        continue;
                    } else {
                        // update children list of parent
                        parent.children.push(index);
                        self.vertices.insert(*parent_index, parent);
                    }
                }
            }

            self.vertices.insert(
                index,
                Vertex {
                    ancestors: v.2,
                    children: vec![],
                    data: v.0,
                    depth: v.1,
                    has_pruned: false,
                },
            );
            self.deepest.insert(VertexKey { depth: v.1, index });
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
    pub fn get_deepest(&self) -> Option<Vertex<T>> {
        match self.deepest.get_max() {
            Some(k) => self.vertices.get(&k.index).map(|vertex| vertex.clone()),
            None => None,
        }
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: u32) -> Option<Vertex<T>> {
        self.vertices.get(&index).map(|vertex| vertex.clone())
    }

    pub fn prune_vertex(&mut self, index: u32) {
        if let Some(vertex) = self.vertices.get(&index) {
            if !vertex.has_pruned {
                let mut v = vertex.clone();
                let children = vertex.children.clone();

                // prune children recursively
                for child in children {
                    self.prune_vertex(child);
                }

                // prune vertex
                v.has_pruned = true;
                self.vertices.insert(index, v);
            }
        }
    }
}
