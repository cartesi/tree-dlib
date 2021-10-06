pub use tree_mod::*;
#[allow(clippy::too_many_arguments)]
mod tree_mod {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "Tree was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static TREE_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            serde_json :: from_str ("[{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"uint256\",\"name\":\"_id\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"uint32\",\"name\":\"_parent\",\"type\":\"uint32\"}],\"name\":\"VertexInserted\",\"type\":\"event\"}]") . expect ("invalid abi")
        });
    #[derive(Clone)]
    pub struct Tree<M>(ethers::contract::Contract<M>);
    impl<M> std::ops::Deref for Tree<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers::providers::Middleware> std::fmt::Debug for Tree<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Tree))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, M: ethers::providers::Middleware> Tree<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            let contract = ethers::contract::Contract::new(
                address.into(),
                TREE_ABI.clone(),
                client,
            );
            Self(contract)
        }
        #[doc = "Gets the contract's `VertexInserted` event"]
        pub fn vertex_inserted_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, VertexInsertedFilter>
        {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(
            &self,
        ) -> ethers::contract::builders::Event<M, VertexInsertedFilter>
        {
            self.0.event_with_filter(Default::default())
        }
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers :: contract :: EthEvent,
    )]
    #[ethevent(name = "VertexInserted", abi = "VertexInserted(uint256,uint32)")]
    pub struct VertexInsertedFilter {
        #[ethevent(indexed)]
        pub id: ethers::core::types::U256,
        pub parent: u32,
    }
}
