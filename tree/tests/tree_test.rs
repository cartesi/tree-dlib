use state_fold::{Access, StateFold};
use tree::fold::tree_delegate::TreeFoldDelegate;

use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{U256, U64};
use std::convert::TryFrom;
use std::fs;
use std::sync::Arc;

// $ geth --dev --http --http.api eth,net,web3
static HTTP_URL: &'static str = "http://localhost:8545";

/// Tests
#[tokio::test]
#[ignore]
async fn tree_test() {
    // getting address from hardhat deployment file
    let test_tree_address: ethers::types::Address = {
        let s = fs::read_to_string("tests/TestTree.address").unwrap();
        s.parse().unwrap()
    };

    let provider = Arc::new(Provider::<Http>::try_from(HTTP_URL).unwrap());
    let access = Access::new(Arc::clone(&provider), U64::from(0), vec![], 4);
    let delegate = TreeFoldDelegate::new(test_tree_address);
    let fold = StateFold::new(delegate, Arc::new(access), 0);

    let latest_hash = provider
        .get_block(provider.get_block_number().await.unwrap())
        .await
        .unwrap()
        .unwrap()
        .hash
        .unwrap();

    let state = fold
        .get_state_for_block(&U256::from(0), latest_hash)
        .await
        .unwrap()
        .state;

    assert!(
        state.tree.is_some(),
        "TestTree should exist for identifier 0"
    );
    assert_eq!(state.tree.unwrap().size(), 8, "TestTree size should match");
}
