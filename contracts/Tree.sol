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


library TreeLibrary {
    struct Tree {
        Vertex[] vertices;
    }

    struct Vertex {
        uint32[] ancestors; // pointers to ancestors' indices in the vertices array (tree)
        uint32 depth; // depth of the vertex in the tree
        bytes data; // data holding in the vertex
    }

    event VertexInserted(uint32 _vertex);

    /// @notice Insert a vertex to the tree
    /// @param _tree pointer to the tree storage
    /// @param _parent the index of parent vertex in the vertices array (tree)
    /// @param _data data of the new vertex going to hold
    function insertVertex(Tree storage _tree, uint32 _parent, bytes memory _data)
        public
    {
        Vertex memory v;
        if (_tree.vertices.length == 0) {
            // insert the very first vertex into the tree
            v = Vertex(new uint32[](0), 0, _data);
        } else {
            // insert vertex to the tree attaching to another vertex
            require(_parent < _tree.vertices.length, "parent index exceeds current tree size");

            uint32 parentDepth = _tree.vertices[_parent].depth;
            // calculate all ancestors' depths of the new vertex
            uint32[] memory requiredDepths = getRequiredDepths(parentDepth + 1);
            uint32[] memory ancestors = new uint32[](requiredDepths.length);

            // construct the ancestors array by getting index of each ancestor in requiredDepths
            for (uint32 i = 0; i < requiredDepths.length; ++i) {
                ancestors[i] = getAncestorAtDepth(_tree, _parent, requiredDepths[i]);
            }

            v = Vertex(ancestors, parentDepth + 1, _data);
        }

        _tree.vertices.push(v);
        emit VertexInserted(uint32(_tree.vertices.length - 1));
    }

    /// @notice Search an ancestor of a vertex in the tree at a certain depth
    /// @param _tree pointer to the tree storage
    /// @param _vertex the index of the vertex in the vertices array (tree)
    /// @param _depth the depth of the ancestor
    function getAncestorAtDepth(Tree storage _tree, uint32 _vertex, uint32 _depth)
        public view returns (uint32)
    {
        require(_vertex < _tree.vertices.length, "vertex index exceeds current tree size");
        require(_depth <= _tree.vertices[_vertex].depth, "search depth deeper than vertex depth");

        // found ancestor
        if (_depth == _tree.vertices[_vertex].depth) {
            return _vertex;
        }

        uint32 vertex = _vertex;
        uint32[] memory ancestorsOfVertex = _tree.vertices[_vertex].ancestors;
        uint32 ancestorsLength = uint32(ancestorsOfVertex.length);
        // start searching from the oldest ancestor (smallest depth)
        // example: search ancestor at depth d(20, b'0001 0100) from vertex v at depth (176, b'1011 0000)
        //    b'1011 0000 -> b'1010 0000 -> b'1000 0000
        // -> b'0100 0000 -> b'0010 0000 -> b'0001 1000
        // -> b'0001 0100
        // TODO: optimize the search process without touching storage all the time
        for (uint32 ancestorsIndex = ancestorsLength - 1; ancestorsIndex < ancestorsLength; --ancestorsIndex) {
            vertex = ancestorsOfVertex[ancestorsIndex];
            Vertex storage ancestor = _tree.vertices[vertex];

            // stop at the ancestor who's closest to the target depth
            if (ancestor.depth >= _depth) {
                break;
            }
        }

        // recursive the search from the current closest ancestor
        return getAncestorAtDepth(_tree, vertex, _depth);
    }

    function getRequiredDepths(uint32 _depth) private pure returns (uint32[] memory) {
        // parent is always included in the ancestors
        uint32 depth = _depth - 1;
        uint32 count = 1;

        // get count of trailing ones of _depth in the binary representation
        while (depth & 1 > 0) {
            depth = depth >> 1;
            ++count;
        }

        depth = _depth - 1;
        uint32[] memory depths = new uint32[](count);
        uint32 i = 0;

        // construct the depths array by removing the trailing ones from lsb one by one
        // example _depth = b'1100 0000: b'1011 1111 -> b'1011 1110 -> b'1011 1100
        //                            -> b'1011 1000 -> b'1011 0000 -> b'1010 0000
        //                            -> b'1000 0000
        while (i < count) {
            depths[i] = depth;
            depth = depth - (uint32(1) << i);
            ++i;
        }

        return depths;
    }
}
