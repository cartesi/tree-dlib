use crate::tree_lib::Tree;

use super::contracts::tree_contract;

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::{BlockState, StateFoldDelegate},
    utils as fold_utils,
};

use async_trait::async_trait;
use ethers::types::{Address, U256};
use snafu::ResultExt;

/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug)]
pub struct TreeState {
    // call_address is the contract address who owns the library object
    pub caller_address: Address,
    pub identifier: U256,
    pub tree: Option<Tree>,
}

/// Tree StateFold Delegate, which implements `sync` and `fold`.
#[derive(Default)]
pub struct TreeFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for TreeFoldDelegate {
    type InitialState = (U256, Address);
    type Accumulator = TreeState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (identifier, caller_address) = *initial_state;

        let contract = access
            .build_sync_contract(
                caller_address,
                block.number,
                tree_contract::Tree::new,
            )
            .await;

        // Get all inserted events.
        let inserted_events = contract
            .vertex_inserted_filter()
            .topic1(identifier)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for vertex inserted",
            })?;

        let state = compute_state(
            inserted_events,
            TreeState {
                caller_address,
                tree: None,
                identifier,
            },
        )
        .map_err(|e| {
            SyncDelegateError {
                err: format!("Could not update tree state: {}", e),
            }
            .build()
        })?;

        Ok(state)
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let identifier = previous_state.identifier;
        let caller_address = previous_state.caller_address;

        // Check if there was (possibly) some log emited on this block.
        let bloom = block.logs_bloom;
        if !(fold_utils::contains_address(&bloom, &caller_address)
            && fold_utils::contains_topic(&bloom, &identifier))
        {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                caller_address,
                block.hash,
                tree_contract::Tree::new,
            )
            .await;

        // Get all inserted events.
        let inserted_events = contract
            .vertex_inserted_filter()
            .topic1(identifier)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for vertex inserted",
            })?;

        let state = compute_state(inserted_events, previous_state.clone())
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Could not update tree state: {}", e),
                }
                .build()
            })?;

        Ok(state)
    }

    fn convert(&self, state: &BlockState<Self::Accumulator>) -> Self::State {
        state.clone()
    }
}

/// Computes the state from all events emission
fn compute_state(
    events: Vec<tree_contract::VertexInsertedFilter>,
    previous_state: TreeState,
) -> crate::error::Result<TreeState> {
    let tree =
        events
            .into_iter()
            .try_fold(previous_state.tree, |tree, event| {
                tree.unwrap_or_default()
                    .insert_vertex(event.parent)
                    .map(|tree| Some(tree))
            })?;

    Ok(TreeState {
        caller_address: previous_state.caller_address,
        identifier: previous_state.identifier,
        tree,
    })
}
