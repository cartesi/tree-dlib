// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Test Tree
pragma abicoder v2;
pragma solidity ^0.8.0;

import "../Tree.sol";

contract TestTree {
    using Tree for Tree.TreeCtx;
    Tree.TreeCtx t;

    // Duplicate event from Tree
    event VertexInserted(uint256 _parent);

    // event VertexInserted(uint32 _index, Tree.Vertex _vertex);

    constructor() {
        insertVertex(0); // first vertex, the parent index is ignored

        insertVertex(0);
        insertVertex(1);
        insertVertex(2);
        insertVertex(3);
        insertVertex(4);
        insertVertex(5);
        insertVertex(6);
    }

    function insertVertex(uint256 _parent) public {
        t.insertVertex(_parent);
    }

    function getDeepest() public view returns (uint256, uint256) {
        return t.getDeepest();
    }

    function getDepth(uint256 _vertex) public view returns (uint256) {
        return t.getDepth(_vertex);
    }

    // function getVertex(uint32 _vertex)
    //     public
    //     view
    //     returns (Tree.Vertex memory)
    // {
    //     return t.getVertex(_vertex);
    // }

    function getTreeSize() public view returns (uint256) {
        return t.getTreeSize();
    }

    function getAncestorAtDepth(uint256 _vertex, uint256 _depth)
        public
        view
        returns (uint256)
    {
        return t.getAncestorAtDepth(_vertex, _depth);
    }

    function getAncestors(uint32 _vertex)
        public
        view
        returns (uint32[] memory)
    {
        Tree.Vertex storage v = t.getVertex(_vertex);
        uint32[] memory ancestors = new uint32[](v.ancestorsLength);

        for (uint32 i = 0; i < ancestors.length; ++i) {
            ancestors[i] = uint32(t.getAncestor(_vertex, i));
        }

        return ancestors;
    }
}
