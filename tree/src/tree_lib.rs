use async_trait::async_trait;
use state_actor::error::*;
use state_actor::types::*;

use futures::future::join_all;
use std::future::Future;

use ethabi::Token;
use im::{HashMap, OrdSet};
use web3::types::{Bytes, H256, U256, U64};

use std::cmp::Ordering;

/// `index` is the unique identifier to each vertex while `depth` is used for sorting.
/// The deepest vertex is defined as largest `depth`, and smallest `index` when the `depth`s are equal,
/// meaning the vertex is oldest in that `depth`
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq)]
struct VertexKey {
    depth: U256,
    index: U256,
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
    ancestors: Vec<U256>,
    children: Vec<U256>,
    data: T,
    depth: U256,
    has_pruned: bool,
}

#[derive(Clone, Debug)]
pub struct Tree <T>
where
    T: Clone,
{
    vertices: HashMap<U256, Vertex<T>>,
    deepest: OrdSet<VertexKey>,
}

impl<T> Tree <T>
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
        new_indices: &Vec<U256>,
        onchain_get_vertex: impl Fn(U256) -> F,
    ) -> Result<()>
    where
        // (T, depth, ancestors)
        F: Future<Output = Result<(T, U256, Vec<U256>)>>,
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
                Vertex{
                    ancestors: v.2,
                    children: vec![],
                    data: v.0,
                    depth: v.1,
                    has_pruned: false,
                }
            );
            self.deepest.insert(
                VertexKey {
                    depth: v.1,
                    index,
                },
            );
        }

        Ok(())
    }

    /// get ancestor of vertex at depth
    pub fn get_ancestor_at(&self, index: U256, depth: U256) -> Option<Vertex<T>> {
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
            },
            // vertex not found with given index
            None => None,
        }

    }
    
    /// get deepest vertex
    pub fn get_deepest(&self) -> Option<Vertex<T>> {
        match self.deepest.get_max() {
            Some(k) => self.vertices
                        .get(&k.index)
                        .map(|vertex| vertex.clone()),
            None => None,
        }
    }

    /// get vertex by index
    pub fn get_vertex(&self, index: U256) -> Option<Vertex<T>> {
        self.vertices
            .get(&index)
            .map(|vertex| vertex.clone())
    }

    pub fn prune_vertex(&mut self, index: U256) {
        if let Some(vertex) = self.vertices
            .get(&index) {
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


/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug)]
pub struct TreeState {
    pub state: Tree<Bytes>,
}

/// Tree StateActor Delegate, which implements `sync` and `fold`.
pub struct TreeStateActorDelegate {}

#[async_trait]
impl StateActorDelegate for TreeStateActorDelegate {
    type State = TreeState;

    async fn sync<T: SyncProvider>(
        &self,
        block_number: U64,
        provider: &T,
    ) -> Result<Self::State> {
        let block_hash = provider.get_block_hash(block_number).await?;
        let mut state = Tree::new();

        // Get all inserted events.
        let inserted_events: Vec<U256> = {
            let inserted_events_fut = provider.get_events_until(
                "Tree",
                "VertexInserted",
                (),
                (),
                (),
                block_number,
            );

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<U256> =
                inserted_events_res?.into_iter().map(|x| x.ret).collect();

            inserted_events
        };

        // Add all previous vertices to the state
        state
            .add_vertices(&inserted_events, |x| {
                onchain_get_vertex(x, block_hash, provider)
            })
            .await?;

        Ok(TreeState {
            state,
        })
    }

    async fn fold<T: FoldProvider>(
        &self,
        previous_state: &Self::State,
        block_hash: H256,
        provider: &T,
    ) -> Result<Self::State> {
        let mut new_state = previous_state.clone();

        // Get all inserted events.
        let inserted_events: Vec<U256> = {
            let inserted_events_fut = provider.get_events_at_block(
                "Tree",
                "VertexInserted",
                (),
                (),
                (),
                block_hash,
            );

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<U256> =
                inserted_events_res?.into_iter().map(|x| x.ret).collect();

            inserted_events
        };

        // Add new vertex to the state
        new_state
            .state
            .add_vertices(&inserted_events, |x| {
                onchain_get_vertex(x, block_hash, provider)
            })
            .await?;

        Ok(new_state)
    }
}

pub async fn onchain_get_vertex<T: FoldProvider>(
    index: U256,
    block_hash: H256,
    provider: &T,
) -> Result<(Bytes, U256, Vec<U256>)> {
    /*
    struct Vertex {
            uint32[] ancestors; // pointers to ancestors' indices in the vertices array (tree)
            uint32 depth; // depth of the vertex in the tree
            bytes data; // data holding in the vertex
        }
    */
    let v = match provider
        .query(
            "Tree",
            "getVertex",
            index,
            None,
            block_hash,
        )
        .await?
    {
        Token::Tuple(t) => t,
        _ => {
            return BlockchainInconsistent {
                err: "Unrecognized vertex structure",
            }
            .fail()
        },
    };

    let (ancestors, depth, data): (Vec<U256>, U256, Bytes) = {
        let a: Vec<U256> = v[0].clone().to_array().unwrap()
            .into_iter()
            .map(|x| x.to_uint().unwrap())
            .collect();
        let d = v[1].clone().to_uint().unwrap();
        let b = v[2].clone().to_bytes().unwrap();
        (a, d, Bytes(b))
    };
    
    Ok((data, depth, ancestors))
}
