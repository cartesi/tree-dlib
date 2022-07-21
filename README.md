# Tree DLib

This repository contains the on-chain and off-chain pieces that are used to deploy, launch and interact with Cartesi Tree library.

## On-chain Tree library

Designed a skip list data structure that is efficient in speed and gas cost for insert/search operation. Below are explanation of the main functions.

- `insertVertex`: Add a new vertex to the tree with the parent vertex id specified. The parent id will be ignored if the tree is empty. Upon success, the id of newly inserted vertex will be returned.
- `getDeepest`: Get the deepest vertex id and its depth of the tree.
- `getAncestorAtDepth`: Get the ancestor id of a vertex at a given depth.

The tree is designed to hold up to `2 ** 32 - 1` of vertices. The use of `uint256` is for better gas effeciency. The library user should make sure all `id` and `depth` parameters don't exceed `2 ** 31 - 1`, otherwise the transaction will be reverted. For example usage please refer to [TestTree](contracts/test/TestTree.sol).

## Off-chain Tree

[WIP]

## Contributing

Thank you for your interest in Cartesi! Head over to our [Contributing Guidelines](CONTRIBUTING.md) for instructions on how to sign our Contributors Agreement and get started with Cartesi!

Please note we have a [Code of Conduct](CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## Authors

* *Stephen Chen*

## License

The tree-dlib repository and all contributions are licensed under
[APACHE 2.0](https://www.apache.org/licenses/LICENSE-2.0). Please review our [LICENSE](LICENSE) file.

## Acknowledgments

* Original work
