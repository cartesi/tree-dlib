pub use testtree_mod::*;
#[allow(clippy::too_many_arguments)]
mod testtree_mod {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    use ethers::{
        contract::{
            self as ethers_contract,
            builders::{ContractCall, Event},
            Contract, Lazy,
        },
        core::{
            self as ethers_core,
            abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
            types::*,
        },
        providers::{self as ethers_providers, Middleware},
    };
    #[doc = "TestTree was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static TESTTREE_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
        ethers_contract::Lazy::new(|| {
            serde_json :: from_str ("[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"uint256\",\"name\":\"_id\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"uint32\",\"name\":\"_parent\",\"type\":\"uint32\"}],\"name\":\"VertexInserted\",\"type\":\"event\"},{\"inputs\":[{\"internalType\":\"uint32\",\"name\":\"_vertex\",\"type\":\"uint32\"},{\"internalType\":\"uint32\",\"name\":\"_depth\",\"type\":\"uint32\"}],\"name\":\"getAncestorAtDepth\",\"outputs\":[{\"internalType\":\"uint32\",\"name\":\"\",\"type\":\"uint32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getTreeSize\",\"outputs\":[{\"internalType\":\"uint32\",\"name\":\"\",\"type\":\"uint32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint32\",\"name\":\"_vertex\",\"type\":\"uint32\"}],\"name\":\"getVertex\",\"outputs\":[{\"components\":[{\"internalType\":\"uint32[]\",\"name\":\"ancestors\",\"type\":\"uint32[]\"},{\"internalType\":\"uint32\",\"name\":\"depth\",\"type\":\"uint32\"}],\"internalType\":\"struct Tree.Vertex\",\"name\":\"\",\"type\":\"tuple\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint32\",\"name\":\"_parent\",\"type\":\"uint32\"}],\"name\":\"insertVertex\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]") . expect ("invalid abi")
        });
    #[derive(Clone)]
    pub struct TestTree<M>(ethers_contract::Contract<M>);
    impl<M> std::ops::Deref for TestTree<M> {
        type Target = ethers_contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers_providers::Middleware> std::fmt::Debug for TestTree<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(TestTree))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, M: ethers_providers::Middleware> TestTree<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            let contract =
                ethers_contract::Contract::new(address.into(), TESTTREE_ABI.clone(), client);
            Self(contract)
        }
        #[doc = "Calls the contract's `getAncestorAtDepth` (0x370841a4) function"]
        pub fn get_ancestor_at_depth(
            &self,
            vertex: u32,
            depth: u32,
        ) -> ethers_contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([55, 8, 65, 164], (vertex, depth))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getTreeSize` (0xeed11836) function"]
        pub fn get_tree_size(&self) -> ethers_contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([238, 209, 24, 54], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getVertex` (0xa62402c3) function"]
        pub fn get_vertex(
            &self,
            vertex: u32,
        ) -> ethers_contract::builders::ContractCall<M, (Vec<u32>, u32)> {
            self.0
                .method_hash([166, 36, 2, 195], vertex)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `insertVertex` (0x802a6b30) function"]
        pub fn insert_vertex(&self, parent: u32) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([128, 42, 107, 48], parent)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `VertexInserted` event"]
        pub fn vertex_inserted_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, VertexInsertedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers_contract::builders::Event<M, VertexInsertedFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq, ethers_contract :: EthEvent)]
    #[ethevent(name = "VertexInserted", abi = "VertexInserted(uint256,uint32)")]
    pub struct VertexInsertedFilter {
        #[ethevent(indexed)]
        pub id: ethers_core::types::U256,
        pub parent: u32,
    }
}
