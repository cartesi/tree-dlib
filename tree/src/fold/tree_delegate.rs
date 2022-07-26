use crate::error::*;
use crate::tree_lib::Tree;

use super::contracts::tree_contract;

use offchain_core::types::Block;
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};

use async_trait::async_trait;
use ethers::providers::Middleware;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::sync::Arc;

/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TreeState {
    // call_address is the contract address who owns the library object
    pub caller_address: Address,
    pub identifier: U256,
    pub tree: Option<Tree>,
}

#[async_trait]
impl Foldable for TreeState {
    type InitialState = (U256, Address);
    type Error = Error;

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M>,
        access: Arc<SyncMiddleware<M>>,
    ) -> std::result::Result<Self, Self::Error> {
        let (identifier, caller_address) = *initial_state;

        compute_state(access, caller_address, identifier, None).await
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        _env: &StateFoldEnvironment<M>,
        access: Arc<FoldMiddleware<M>>,
    ) -> std::result::Result<Self, Self::Error> {
        let identifier = previous_state.identifier;
        let caller_address = previous_state.caller_address;

        // Check if there was (possibly) some log emited on this block.
        let bloom = block.logs_bloom;
        if !(fold_utils::contains_address(&bloom, &caller_address)
            && fold_utils::contains_topic(&bloom, &identifier))
        {
            return Ok(previous_state.clone());
        }

        compute_state(
            access,
            caller_address,
            identifier,
            previous_state.tree.clone(),
        )
        .await
    }
}

/// Computes the state from all events emission
async fn compute_state<M: Middleware + 'static>(
    access: Arc<M>,
    caller_address: Address,
    identifier: U256,
    previous_tree: Option<Tree>,
) -> crate::error::Result<TreeState> {
    let contract = tree_contract::Tree::new(caller_address, access);

    // Get all inserted events.
    let inserted_events = contract
        .vertex_inserted_filter()
        .topic1(identifier)
        .query()
        .await
        .map_err(|e| e.into())
        .context(TreeUnavailable {
            err: format!("Error querying for vertex inserted"),
        })?;

    let tree = inserted_events.into_iter().try_fold(
        previous_tree,
        |tree, event| {
            tree.unwrap_or_default()
                .insert_vertex(event.parent)
                .map(|tree| Some(tree))
        },
    )?;

    Ok(TreeState {
        caller_address,
        identifier,
        tree,
    })
}
