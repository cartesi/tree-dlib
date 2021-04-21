use dispatcher::state_fold::{Access, StateFold};
// use dispatcher::types::*;
use tree::fold::tree_delegate::TreeFoldDelegate;

// use ethabi::Token;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{U256, U64};
use std::convert::TryFrom;
use std::fs;
use std::sync::Arc;
// use web3::types::{BlockId, BlockNumber, Bytes, TransactionRequest, U256};

static CONTRACT: &'static str = "TestTree";

// $ geth --dev --http --http.api eth,net,web3
static HTTP_URL: &'static str = "http://localhost:8545";

/// Tests
#[tokio::test]
#[ignore]
async fn tree_test() {
    // @dev can we add this function to dispatcher-util?
    // getting address from hardhat deployment file
    let test_tree_address: ethers::types::Address = {
        let s = fs::read_to_string("tests/deployment.json").unwrap();

        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        let address_str = &json["contracts"][CONTRACT]["address"];

        let address_str = serde_json::to_string(address_str).unwrap();

        let address_str = {
            // Remove leading quotes and '0x', and remove trailing quotes
            let leading_offset = 3;
            let len = address_str.len() - 1;

            &address_str[leading_offset..len]
        };

        address_str.parse().unwrap()
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
