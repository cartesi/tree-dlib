use crate::tree_lib::Tree;
use async_trait::async_trait;
use state_fold::error::*;
use state_fold::types::*;

use im::HashMap;
use web3::types::{H256, U256, U64};

/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug)]
pub struct TreeState {
    // There's an extra `U256` identifier in case there're more than one Tree
    // in the caller contract. Similar to the instantiator pattern.
    pub state: HashMap<U256, Tree>,
}

pub type TreeStateFold = state_fold::StateFold<
    (),
    TreeState,
    BlockState<TreeState>,
    TreeStateFoldDelegate,
    state_fold::provider::Factory,
>;

/// Tree StateFold Delegate, which implements `sync` and `fold`.
pub struct TreeStateFoldDelegate {
    contract: String,
}

impl TreeStateFoldDelegate {
    pub fn new(contract: &str) -> Self {
        TreeStateFoldDelegate {
            contract: contract.to_string(),
        }
    }
}

#[async_trait]
impl StateFoldDelegate for TreeStateFoldDelegate {
    type InitialState = ();
    type Accumulator = TreeState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<T: SyncProvider>(
        &self,
        _: &Self::InitialState,
        block_number: U64,
        provider: &T,
    ) -> Result<Self::Accumulator> {
        // Get all inserted events.
        // event VertexInserted(uint256 _id, uint32 _parent);
        let parsed_events: Vec<(U256, u32)> = {
            provider
                .get_events_until(&self.contract, "VertexInserted", (), (), (), block_number)
                .await
                .and_then(|mut events| {
                    state_fold::util::sort_events(&mut events);
                    Ok(events)
                })?
                .into_iter()
                .map(|x: Event<(U256, U256)>| (x.ret.0, x.ret.1.as_u32()))
                .collect()
        };

        // Add all previous vertices to the state
        let state = parsed_events
            .into_iter()
            .try_fold(
                TreeState {
                    state: HashMap::new(),
                },
                |state, event| {
                    // Update Tree with given U256 identifier
                    state
                        .state
                        .get(&event.0)
                        .unwrap_or(&Tree::new())
                        .clone()
                        .insert_vertex(event.1)
                        .map(|tree| TreeState {
                            state: state.state.update(event.0, tree),
                        })
                },
            )
            .map_err(|e| {
                BlockchainTemporaryError {
                    err: format!("Cannot insert vertex to tree state {}", e),
                }
                .build()
            })?;

        Ok(state)
    }

    async fn fold<T: FoldProvider>(
        &self,
        previous_state: &Self::Accumulator,
        block_hash: H256,
        provider: &T,
    ) -> Result<Self::Accumulator> {
        // Get all inserted events.
        // event VertexInserted(uint256 _id, uint32 _parent);
        let parsed_events: Vec<(U256, u32)> = {
            provider
                .get_events_at_block(&self.contract, "VertexInserted", (), (), (), block_hash)
                .await
                .and_then(|mut events| {
                    state_fold::util::sort_events(&mut events);
                    Ok(events)
                })?
                .into_iter()
                .map(|x: Event<(U256, U256)>| (x.ret.0, x.ret.1.as_u32()))
                .collect()
        };

        let state = parsed_events
            .into_iter()
            .try_fold(previous_state.clone(), |state, event| {
                // Update Tree with given U256 identifier
                state
                    .state
                    .get(&event.0)
                    .unwrap_or(&Tree::new())
                    .clone()
                    .insert_vertex(event.1)
                    .map(|tree| TreeState {
                        state: state.state.update(event.0, tree),
                    })
            })
            .map_err(|e| {
                BlockchainTemporaryError {
                    err: format!("Cannot insert vertex to tree state {}", e),
                }
                .build()
            })?;

        Ok(state)
    }

    fn convert(&self, state: &BlockState<Self::Accumulator>) -> Self::State {
        state.clone()
    }
}
