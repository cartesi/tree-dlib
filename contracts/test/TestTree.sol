// Copyright 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.


/// @title Library for Tree service
pragma solidity ^0.7.0;
pragma experimental ABIEncoderV2;

import "../Tree.sol";


contract TestTree {
    using TreeLibrary for TreeLibrary.Tree;
    TreeLibrary.Tree t;

    event VertexInserted(uint32 _vertex);

    constructor() {
        t.insertVertex(0, "Vertex 0"); // first vertex, the parent index is ignored

        t.insertVertex(0, "Vertex 1");
        t.insertVertex(1, "Vertex 2");
        t.insertVertex(2, "Vertex 3");
        t.insertVertex(3, "Vertex 4");
        t.insertVertex(4, "Vertex 5");
        t.insertVertex(5, "Vertex 6");
        t.insertVertex(6, "Vertex 7");
    }

    function insertVertex(uint32 _parent, bytes memory _data) public {
        t.insertVertex(_parent, _data);
    }

    function getVertex(uint32 _vertex) public view returns (TreeLibrary.Vertex memory) {
        require(_vertex < t.vertices.length, "vertex index exceeds current tree size");

        return t.vertices[_vertex];
    }

    function getTreeSize() public view returns (uint32) {
        return uint32(t.vertices.length);
    }

}
