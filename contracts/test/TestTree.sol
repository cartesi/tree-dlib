// Copyright 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Test TreeLibrary
pragma solidity ^0.7.0;
pragma experimental ABIEncoderV2;

import "../Tree.sol";

contract TestTree {
    using TreeLibrary for TreeLibrary.Tree;
    TreeLibrary.Tree t;

    event VertexInserted(uint32 _index, uint32 _parent, uint32 _depth);

    // event VertexInserted(uint32 _index, TreeLibrary.Vertex _vertex);

    constructor() {
        t.insertVertex(0); // first vertex, the parent index is ignored

        t.insertVertex(0);
        t.insertVertex(1);
        t.insertVertex(2);
        t.insertVertex(3);
        t.insertVertex(4);
        t.insertVertex(5);
        t.insertVertex(6);
    }

    function insertVertex(uint32 _parent) public {
        t.insertVertex(_parent);
    }

    function getVertex(uint32 _vertex)
        public
        view
        returns (TreeLibrary.Vertex memory)
    {
        return t.getVertex(_vertex);
    }

    function getTreeSize() public view returns (uint32) {
        return t.getTreeSize();
    }

    function getAncestorAtDepth(uint32 _vertex, uint32 _depth)
        public
        view
        returns (uint32)
    {
        return t.getAncestorAtDepth(_vertex, _depth);
    }
}
