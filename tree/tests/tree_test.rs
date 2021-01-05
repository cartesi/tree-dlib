use async_trait::async_trait;
use dispatcher::Actor;
use state_actor::error::*;
use state_actor::types::*;
use tree::tree_lib::{Tree, Vertex};

use ethabi::Token;
use web3::types::{Bytes, TransactionRequest, H256, U256, U64};

use futures::future::join_all;
use snafu::ResultExt;
use tokio::sync::{mpsc, watch};

// $ geth --dev --http --http.api eth,net,web3
static HTTP_URL: &'static str = "http://localhost:8545";
static WS_URL: &'static str = "ws://localhost:8546";

/// Actor message
enum Message {
    TreeStateActorMessage(BlockState<TreeState>),
}

fn wrap_message(state: BlockState<TreeState>) -> Message {
    Message::TreeStateActorMessage(state)
}

fn unwrap_state(message: Message) -> TreeState {
    let Message::TreeStateActorMessage(state) = message;
    state.state
}

/// Tree dlib state, to be passed to and returned by fold.
#[derive(Clone, Debug)]
pub struct TreeState {
    pub state: Tree<Vec<u8>>,
}

/// Tree StateActor Delegate, which implements `sync` and `fold`.
pub struct TreeStateActorDelegate {}

#[async_trait]
impl StateActorDelegate for TreeStateActorDelegate {
    type State = TreeState;

    async fn sync<T: SyncProvider>(&self, block_number: U64, provider: &T) -> Result<Self::State> {
        let block_hash = provider.get_block_hash(block_number).await?;
        let mut state = Tree::new();

        // Get all inserted events.
        let inserted_events: Vec<u32> = {
            let inserted_events_fut =
                provider.get_events_until("Tree", "VertexInserted", (), (), (), block_number);

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<u32> = inserted_events_res?
                .into_iter()
                .map(|x: Event<U256>| x.ret.as_u32())
                .collect();

            inserted_events
        };

        let mut futures = vec![];

        for index in inserted_events {
            let future = {
                let idx = index.clone();
                let vertex = onchain_get_vertex(index, block_hash, provider).await?;
                async move { (idx, vertex) }
            };
            futures.push(future);
        }
        let vertices = join_all(futures).await;

        // Add all previous vertices to the state
        state.add_vertices(vertices).await?;

        Ok(TreeState { state })
    }

    async fn fold<T: FoldProvider>(
        &self,
        previous_state: &Self::State,
        block_hash: H256,
        provider: &T,
    ) -> Result<Self::State> {
        let mut new_state = previous_state.clone();

        // Get all inserted events.
        let inserted_events: Vec<u32> = {
            let inserted_events_fut =
                provider.get_events_at_block("Tree", "VertexInserted", (), (), (), block_hash);

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<u32> = inserted_events_res?
                .into_iter()
                .map(|x: Event<U256>| x.ret.as_u32())
                .collect();

            inserted_events
        };

        let mut futures = vec![];

        for index in inserted_events {
            let future = {
                let idx = index.clone();
                let vertex = onchain_get_vertex(index, block_hash, provider).await?;
                async move { (idx, vertex) }
            };
            futures.push(future);
        }
        let vertices = join_all(futures).await;

        // Add new vertex to the state
        new_state.state.add_vertices(vertices).await?;

        Ok(new_state)
    }
}

pub async fn onchain_get_vertex<T: FoldProvider>(
    index: u32,
    block_hash: H256,
    provider: &T,
) -> Result<Vertex<Vec<u8>>> {
    /*
    struct Vertex {
            uint32[] ancestors; // pointers to ancestors' indices in the vertices array (tree)
            uint32 depth; // depth of the vertex in the tree
            bytes data; // data holding in the vertex
        }
    */
    let v = match provider
        .query("Tree", "getVertex", index, None, block_hash)
        .await?
    {
        Token::Tuple(t) => t,
        _ => {
            return BlockchainInconsistent {
                err: "Unrecognized vertex structure",
            }
            .fail()
        }
    };

    let (ancestors, depth, data): (Vec<u32>, u32, Vec<u8>) = {
        let a_token = v
            .get(0)
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot get vertex ancestors array",
            })?
            .clone()
            .to_array()
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot convert vertex ancestors array",
            })?;
        let a = {
            let mut a = vec![];
            for ancestor in a_token {
                a.push(
                    ancestor
                        .to_uint()
                        .ok_or(snafu::NoneError)
                        .context(BlockchainInconsistent {
                            err: "Cannot get vertex ancestor",
                        })?
                        .as_u32(),
                )
            }
            a
        };
        let d = v
            .get(1)
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot get vertex depth",
            })?
            .clone()
            .to_uint()
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot convert vertex depth",
            })?
            .as_u32();

        let b = v
            .get(2)
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot get vertex data",
            })?
            .clone()
            .to_bytes()
            .ok_or(snafu::NoneError)
            .context(BlockchainInconsistent {
                err: "Cannot convert vertex data",
            })?;
        (a, d, b)
    };

    Ok(Vertex {
        ancestors: ancestors,
        children: vec![],
        data: data,
        depth: depth,
        has_pruned: false,
    })
}

/// Tests
#[tokio::test]
#[ignore]
async fn tree_state_test() {
    let web3 = utils::new_web3_http(HTTP_URL);
    let accounts = web3.eth().accounts().await.unwrap();
    let delegate = TreeStateActorDelegate {};

    let (tree_actor, _subscriber_kill_switch, contract_data) =
        state_actor::helper::create_dapp_state_actor(
            vec![("../deployments/localhost/TestTree.json".into(), "Tree")],
            delegate,
            1,
            HTTP_URL,
            WS_URL,
            wrap_message,
        )
        .await;

    let (msg_tx, mut messages) = mpsc::unbounded_channel();
    let (kill_switch, kill_rx) = watch::channel(dispatcher::KillSwitchStatus::Normal);

    let handle = tree_actor.start(msg_tx, kill_rx).await.unwrap();
    let test_data = "test vertex from rust".as_bytes().to_vec();

    let input_tokens = vec![Token::Uint(U256::from(6)), Token::Bytes(test_data.clone())];

    let data = contract_data[0]
        .abi
        .function("insertVertex")
        .unwrap()
        .encode_input(&input_tokens)
        .unwrap();

    let _tx = web3
        .send_transaction_with_confirmation(
            TransactionRequest {
                from: accounts[0],
                to: Some(contract_data[0].address.clone()),
                gas: None,
                gas_price: None,
                value: None,
                nonce: None,
                data: Some(Bytes(data.clone())),
                condition: None,
            },
            std::time::Duration::from_secs(2),
            0, // do not wait for confirmation
        )
        .await
        .unwrap();

    let mut state = unwrap_state(messages.recv().await.unwrap());

    let v7 = state.state.get_vertex(7).unwrap();
    let v8 = state.state.get_vertex(8).unwrap();

    assert_eq!(v8.ancestors, [6]);
    assert_eq!(v8.children.len(), 0);
    assert_eq!(v8.data, test_data);
    assert_eq!(v8.has_pruned, false);

    let deepest = state.state.get_deepest().unwrap();
    assert_eq!(v7, deepest.1);

    state.state.prune_vertex(6);
    let pruned_v8 = state.state.get_vertex(8).unwrap();
    assert_eq!(pruned_v8.has_pruned, true);

    // kill actor
    kill_switch
        .broadcast(dispatcher::KillSwitchStatus::Shutdown)
        .unwrap();

    let ret = handle.await.unwrap();
    assert_eq!(ret.unwrap(), ());
}
