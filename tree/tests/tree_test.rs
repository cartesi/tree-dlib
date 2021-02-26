use dispatcher_types::*;
use tree::tree_fold::TreeStateFoldDelegate;

use ethabi::Token;
use web3::types::{BlockId, BlockNumber, Bytes, TransactionRequest, U256};

use std::sync::Arc;

pub static CONTRACT: &'static str = "TestTree";

// $ geth --dev --http --http.api eth,net,web3
static HTTP_URL: &'static str = "http://localhost:8545";

/// Tests
#[tokio::test]
#[ignore]
async fn tree_state_test() {
    let web3_factory = web3_factory::Web3Factory::new();
    let contract_data = ContractData::new_from_hardhat_export(
        &std::path::PathBuf::from("tests/deployment.json"),
        &vec![CONTRACT],
    )
    .unwrap();
    let fold_factory = Arc::new(
        state_fold::provider::Factory::new(
            HTTP_URL.to_string(),
            Arc::clone(&web3_factory),
            std::time::Duration::from_millis(10),
            1,
            1,
            contract_data.clone(),
        )
        .unwrap(),
    );

    let delegate = TreeStateFoldDelegate::new(CONTRACT);
    let fold = state_fold::StateFold::new(
        delegate,
        fold_factory,
        1,
        4,
        std::time::Duration::from_millis(10),
    );

    let input_tokens = vec![Token::Uint(U256::from(6))];

    let data = contract_data[0]
        .abi
        .function("insertVertex")
        .unwrap()
        .encode_input(&input_tokens)
        .unwrap();

    let web3 = utils::new_web3_http(HTTP_URL);
    let accounts = web3.eth().accounts().await.unwrap();
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

    let latest_block_hash = web3
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap()
        .unwrap()
        .hash
        .unwrap();

    let state_message = fold
        .get_state_for_block(&(), latest_block_hash)
        .await
        .unwrap();

    let state = state_message
        .state
        .state
        .get(&U256::from(0))
        .unwrap()
        .clone();

    let v7 = state.get_vertex(7);
    let v8 = state.get_vertex(8);
    let v0 = state.get_vertex_rc(0);
    let v2 = state.get_vertex_rc(2);
    let v6 = state.get_vertex_rc(6);

    assert!(v7.is_some(), "Vertex7 should exist");
    assert!(v8.is_some(), "Vertex8 should exist");
    assert_eq!(
        v8.and_then(|v| v.parent.clone()),
        v6,
        "Parent of Vertex8 should be 6"
    );

    let deepest = state
        .get_deepest()
        .and_then(|deepest| state.get_vertex(deepest));

    assert!(deepest.is_some(), "Deepest vertex should exist");
    assert_eq!(deepest, v7, "Deepest vertex should be Vertex7");

    // Should return ancestors successfully
    assert_eq!(
        state.get_ancestor_rc_at(7, 0).ok(),
        v0,
        "Ancestor at depth 0 should exist"
    );
    assert_eq!(
        state.get_ancestor_rc_at(7, 6).ok(),
        v6,
        "Ancestor at depth 6 should exist"
    );
    assert_eq!(
        state.get_ancestor_rc_at(8, 6).ok(),
        v6,
        "Ancestor at depth 6 should exist"
    );
    assert_eq!(
        state.get_ancestor_rc_at(8, 2).ok(),
        v2,
        "Ancestor at depth 2 should exist"
    );
    // Should fail to get ancestor
    assert!(
        state.get_ancestor_rc_at(8, 20).is_err(),
        "Ancestor at depth 20 shouldn't exist"
    );
}
