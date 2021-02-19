use crate::tree_lib::Tree;
use async_trait::async_trait;
use state_fold::error::*;
use state_fold::types::*;

use web3::types::{H256, U256, U64};

/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug)]
pub struct TreeState {
    pub state: Tree,
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
        // let block_hash = provider.get_block_hash(block_number).await?;

        // Get all inserted events.
        // event VertexInserted(uint32 _index, uint32 _parent, uint32 _depth, bytes _data);
        let parsed_events: Vec<(u32, u32, u32)> = {
            let inserted_events_fut = provider.get_events_until(
                &self.contract,
                "VertexInserted",
                (),
                (),
                (),
                block_number,
            );

            let sorted_events = inserted_events_fut.await.and_then(|mut events| {
                state_fold::util::sort_events(&mut events);
                Ok(events)
            })?;

            let parsed_events: Vec<(u32, u32, u32)> = sorted_events
                .into_iter()
                .map(|x: Event<(U256, U256, U256)>| {
                    (x.ret.0.as_u32(), x.ret.1.as_u32(), x.ret.2.as_u32())
                })
                .collect();

            parsed_events
        };

        // Add all previous vertices to the state
        let state = parsed_events
            .into_iter()
            .try_fold(Tree::new(), |state, event| state.insert_vertex(event))
            .map_err(|e| {
                BlockchainTemporaryError {
                    err: format!("Cannot insert vertex to tree state {}", e),
                }
                .build()
            })?;

        Ok(TreeState { state })
    }

    async fn fold<T: FoldProvider>(
        &self,
        previous_state: &Self::Accumulator,
        block_hash: H256,
        provider: &T,
    ) -> Result<Self::Accumulator> {
        let new_state = previous_state.state.clone();

        // Get all inserted events.
        // event VertexInserted(uint32 _index, uint32 _parent, uint32 _depth, bytes _data);
        let parsed_events: Vec<(u32, u32, u32)> = {
            let inserted_events_fut = provider.get_events_at_block(
                &self.contract,
                "VertexInserted",
                (),
                (),
                (),
                block_hash,
            );

            let sorted_events = inserted_events_fut.await.and_then(|mut events| {
                state_fold::util::sort_events(&mut events);
                Ok(events)
            })?;

            let parsed_events: Vec<(u32, u32, u32)> = sorted_events
                .into_iter()
                .map(|x: Event<(U256, U256, U256)>| {
                    (x.ret.0.as_u32(), x.ret.1.as_u32(), x.ret.2.as_u32())
                })
                .collect();

            parsed_events
        };

        let state = parsed_events
            .into_iter()
            .try_fold(new_state, |state, event| state.insert_vertex(event))
            .map_err(|e| {
                BlockchainTemporaryError {
                    err: format!("Cannot insert vertex to tree state {}", e),
                }
                .build()
            })?;

        Ok(TreeState { state })
    }

    fn convert(&self, state: &BlockState<Self::Accumulator>) -> Self::State {
        state.clone()
    }
}
