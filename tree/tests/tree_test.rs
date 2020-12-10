use dispatcher::Actor;
use state_actor::types;
use tree::tree_lib::{TreeState, TreeStateActorDelegate};

use ethabi::Token;
use web3::types::{Bytes, TransactionRequest, U256};

use tokio::sync::{mpsc, watch};

// $ geth --dev --http --http.api eth,net,web3
static HTTP_URL: &'static str = "http://localhost:8545";
static WS_URL: &'static str = "ws://localhost:8546";

/// Actor message
enum Message {
    TreeStateActorMessage(types::BlockState<TreeState>),
}

fn wrap_message(state: types::BlockState<TreeState>) -> Message {
    Message::TreeStateActorMessage(state)
}

fn unwrap_state(message: Message) -> TreeState {
    let Message::TreeStateActorMessage(state) = message;
    state.state
}

/// Tests
#[tokio::test]
#[ignore]
async fn tree_state_test() {

    let web3 = utils::new_web3_http(HTTP_URL);
    let accounts = web3.eth().accounts().await.unwrap();
    let delegate = TreeStateActorDelegate{};

    let (
        tree_actor,
        _subscriber_kill_switch,
        contract_data
    ) = state_actor::helper::create_dapp_state_actor(
        vec![
            ("../deployments/localhost/TestTree.json".into(),"Tree")
        ],
        delegate, 
        1,
        HTTP_URL,
        WS_URL,
        wrap_message,
    ).await;

    let (msg_tx, mut messages) = mpsc::unbounded_channel();
    let (kill_switch, kill_rx) =
        watch::channel(dispatcher::KillSwitchStatus::Normal);

    let handle = tree_actor.start(msg_tx, kill_rx).await.unwrap();
    
    let input_tokens = vec![
        Token::Uint(U256::from(6)),
        Token::Bytes("test vertex from rust".as_bytes().to_vec()),
    ];

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

    println!("Vertex6 before: {:?}", state.state.get_vertex(U256::from(6)));
    println!("Vertex7 before: {:?}", state.state.get_vertex(U256::from(7)));
    println!("Vertex8 before: {:?}", state.state.get_vertex(U256::from(8)));
    // println!("Deepest: {:?}", state.state.get_deepest());
    // println!("TreeState: {:?}", state);

    state.state.prune_vertex(U256::from(6));
    println!("Vertex6 after: {:?}", state.state.get_vertex(U256::from(6)));
    println!("Vertex7 after: {:?}", state.state.get_vertex(U256::from(7)));
    println!("Vertex8 after: {:?}", state.state.get_vertex(U256::from(8)));

    // kill actor
    kill_switch
        .broadcast(dispatcher::KillSwitchStatus::Shutdown)
        .unwrap();

    let ret = handle.await.unwrap();
    assert_eq!(ret.unwrap(), ());
}
