use async_trait::async_trait;
use dispatcher::Actor;
use state_actor::error::*;
use state_actor::types::*;
use tree::tree_lib::{Tree, Vertex};

use ethabi::Token;
use web3::types::{Bytes, TransactionRequest, H256, U256, U64};

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
        // let block_hash = provider.get_block_hash(block_number).await?;
        let mut state = Tree::new();

        // Get all inserted events.
        // event VertexInserted(uint32 _index, uint32 _parent, uint32 _depth, bytes _data);
        let inserted_events: Vec<(u32, u32, u32, Vec<u8>)> = {
            let inserted_events_fut =
                provider.get_events_until("Tree", "VertexInserted", (), (), (), block_number);

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<(u32, u32, u32, Vec<u8>)> = inserted_events_res?
                .into_iter()
                .map(|x: Event<(U256, U256, U256, Bytes)>| {
                    (
                        x.ret.0.as_u32(),
                        x.ret.1.as_u32(),
                        x.ret.2.as_u32(),
                        x.ret.3 .0.to_vec(),
                    )
                })
                .collect();

            inserted_events
        };

        let vertices: Vec<(u32, Vertex<Vec<u8>>)> = inserted_events
            .into_iter()
            .map(|x| {
                let parent = if (x.0 == 0 && x.1 == 0) {
                    None
                } else {
                    Some(x.1)
                };
                (
                    x.0,
                    Vertex {
                        parent,
                        depth: x.2,
                        data: x.3,
                    },
                )
            })
            .collect();

        // Add all previous vertices to the state
        for vertex in vertices {
            // Add new vertex to the state
            state = state.add_vertex(vertex)?
        }

        Ok(TreeState { state })
    }

    async fn fold<T: FoldProvider>(
        &self,
        previous_state: &Self::State,
        block_hash: H256,
        provider: &T,
    ) -> Result<Self::State> {
        let mut new_state = previous_state.state.clone();

        // Get all inserted events.
        // event VertexInserted(uint32 _index, uint32 _parent, uint32 _depth, bytes _data);
        let inserted_events: Vec<(u32, u32, u32, Vec<u8>)> = {
            let inserted_events_fut =
                provider.get_events_at_block("Tree", "VertexInserted", (), (), (), block_hash);

            let inserted_events_res = inserted_events_fut.await;

            let inserted_events: Vec<(u32, u32, u32, Vec<u8>)> = inserted_events_res?
                .into_iter()
                .map(|x: Event<(U256, U256, U256, Bytes)>| {
                    (
                        x.ret.0.as_u32(),
                        x.ret.1.as_u32(),
                        x.ret.2.as_u32(),
                        x.ret.3 .0.to_vec(),
                    )
                })
                .collect();

            inserted_events
        };

        let vertices: Vec<(u32, Vertex<Vec<u8>>)> = inserted_events
            .into_iter()
            .map(|x| {
                let parent = if (x.0 == 0 && x.1 == 0) {
                    None
                } else {
                    Some(x.1)
                };
                (
                    x.0,
                    Vertex {
                        parent,
                        depth: x.2,
                        data: x.3,
                    },
                )
            })
            .collect();

        for vertex in vertices {
            // Add new vertex to the state
            new_state = new_state.add_vertex(vertex)?
        }

        Ok(TreeState { state: new_state })
    }
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

    web3.send_transaction_with_confirmation(
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

    let state_message = messages.recv().await;

    assert!(
        state_message.is_some(),
        "Should received message from state actor"
    );

    let state = unwrap_state(state_message.unwrap());

    let v7 = state.state.get_vertex(7);
    let v8 = state.state.get_vertex(8);

    assert!(v7.is_some(), "Vertex7 should exist");
    assert!(v8.is_some(), "Vertex8 should exist");
    assert_eq!(
        v8.and_then(|v| v.parent),
        Some(6),
        "Parent of Vertex8 should be 6"
    );
    assert_eq!(
        v8.and_then(|v| Some(v.data.clone())),
        Some(test_data),
        "Data of Vertex8 should match"
    );

    let deepest = state
        .state
        .get_deepest()
        .and_then(|deepest| state.state.get_vertex(deepest));

    assert!(deepest.is_some(), "Deepest vertex should exist");
    assert_eq!(deepest, v7, "Deepest vertex should be Vertex7");

    // Should return ancestors successfully
    assert_eq!(
        state.state.get_ancestor_at(7, 0).ok(),
        Some(0),
        "Ancestor at depth 0 should exist"
    );
    assert_eq!(
        state.state.get_ancestor_at(7, 6).ok(),
        Some(6),
        "Ancestor at depth 6 should exist"
    );
    assert_eq!(
        state.state.get_ancestor_at(8, 6).ok(),
        Some(6),
        "Ancestor at depth 6 should exist"
    );
    assert_eq!(
        state.state.get_ancestor_at(8, 2).ok(),
        Some(2),
        "Ancestor at depth 2 should exist"
    );
    // Should fail to get ancestor
    assert_eq!(
        state.state.get_ancestor_at(8, 20).ok(),
        None,
        "Ancestor at depth 20 shouldn't exist"
    );

    // kill actor
    kill_switch
        .broadcast(dispatcher::KillSwitchStatus::Shutdown)
        .unwrap();

    let ret = handle.await.unwrap();
    assert_eq!(ret.unwrap(), ());
}
