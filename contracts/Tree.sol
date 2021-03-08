// Copyright 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Tree Library
pragma solidity ^0.7.0;
pragma experimental ABIEncoderV2;

library Tree {
    struct TreeCtx {
        Vertex[] vertices;
        uint32 deepestVertex;
        uint32 deepestDepth;
    }

    struct Vertex {
        uint32[] ancestors; // pointers to ancestors' indices in the vertices array (tree)
        uint32 depth; // depth of the vertex in the tree
    }

    // Because Tree is a library, the event is going to be emitted from the caller contract.
    // When there're multiple objects of this library,
    // we need this `_id` to differentiate one from another
    event VertexInserted(uint256 indexed _id, uint32 _parent);

    // event VertexInserted(uint32 _index, Vertex _vertex);

    /// @notice Insert a vertex to the tree
    /// @param _tree pointer to the tree storage
    /// @param _id the identifier to differentiate each tree from caller contract
    /// @param _parent the index of parent vertex in the vertices array (tree)
    /// @return index of the inserted vertex
    function insertVertex(
        TreeCtx storage _tree,
        uint256 _id,
        uint32 _parent
    ) public returns (uint32) {
        Vertex memory v;
        if (_tree.vertices.length == 0) {
            // insert the very first vertex into the tree
            v = Vertex(new uint32[](0), 0);
        } else {
            // insert vertex to the tree attaching to another vertex
            require(
                _parent < _tree.vertices.length,
                "parent index exceeds current tree size"
            );

            uint32 parentDepth = _tree.vertices[_parent].depth;
            // calculate all ancestors' depths of the new vertex
            uint32[] memory requiredDepths = getRequiredDepths(parentDepth + 1);
            uint32[] memory ancestors = new uint32[](requiredDepths.length);

            // construct the ancestors array by getting index of each ancestor in requiredDepths
            for (uint32 i = 0; i < requiredDepths.length; ++i) {
                ancestors[i] = getAncestorAtDepth(
                    _tree,
                    _parent,
                    requiredDepths[i]
                );
            }

            v = Vertex(ancestors, parentDepth + 1);
        }

        uint32 index = getTreeSize(_tree);
        _tree.vertices.push(v);

        if (v.depth > _tree.deepestDepth) {
            _tree.deepestDepth = v.depth;
            _tree.deepestVertex = index;
        }

        emit VertexInserted(_id, _parent);

        return index;
    }

    /// @notice Search an ancestor of a vertex in the tree at a certain depth
    /// @param _tree pointer to the tree storage
    /// @param _vertex the index of the vertex in the vertices array (tree)
    /// @param _depth the depth of the ancestor
    /// @return index of ancestor at depth of _vertex
    function getAncestorAtDepth(
        TreeCtx storage _tree,
        uint32 _vertex,
        uint32 _depth
    ) public view returns (uint32) {
        require(
            _vertex < _tree.vertices.length,
            "vertex index exceeds current tree size"
        );
        require(
            _depth <= _tree.vertices[_vertex].depth,
            "search depth deeper than vertex depth"
        );

        uint32 vertex = _vertex;

        while (_depth != _tree.vertices[vertex].depth) {
            uint32[] memory ancestorsOfVertex =
                _tree.vertices[vertex].ancestors;
            uint32 ancestorsLength = uint32(ancestorsOfVertex.length);
            // start searching from the oldest ancestor (smallest depth)
            // example: search ancestor at depth d(20, b'0001 0100) from vertex v at depth (176, b'1011 0000)
            //    b'1011 0000 -> b'1010 0000 -> b'1000 0000
            // -> b'0100 0000 -> b'0010 0000 -> b'0001 1000
            // -> b'0001 0100

            // given that ancestorsIndex is unsigned, when -1 at 0, it'll underflow and become UINT32_MAX
            // so the continue condition has to be ancestorsIndex < ancestorsLength,
            // can't be ancestorsIndex >= 0
            for (
                uint32 ancestorsIndex = ancestorsLength - 1;
                ancestorsIndex < ancestorsLength;
                --ancestorsIndex
            ) {
                vertex = ancestorsOfVertex[ancestorsIndex];
                Vertex storage ancestor = _tree.vertices[vertex];

                // stop at the ancestor who's closest to the target depth
                if (ancestor.depth >= _depth) {
                    break;
                }
            }
        }

        return vertex;
    }

    /// @notice Get depth of vertex
    /// @param _tree pointer to the tree storage
    /// @param _vertex the index of the vertex in the vertices array (tree)
    function getDepth(TreeCtx storage _tree, uint32 _vertex)
        public
        view
        returns (uint32)
    {
        return getVertex(_tree, _vertex).depth;
    }

    /// @notice Get vertex from the tree
    /// @param _tree pointer to the tree storage
    /// @param _vertex the index of the vertex in the vertices array (tree)
    function getVertex(TreeCtx storage _tree, uint32 _vertex)
        public
        view
        returns (Tree.Vertex memory)
    {
        require(
            _vertex < _tree.vertices.length,
            "vertex index exceeds current tree size"
        );

        return _tree.vertices[_vertex];
    }

    /// @notice Get current tree size
    /// @param _tree pointer to the tree storage
    function getTreeSize(TreeCtx storage _tree) public view returns (uint32) {
        return uint32(_tree.vertices.length);
    }

    /// @notice Get current tree size
    /// @param _tree pointer to the tree storage
    /// @return index number and depth of the deepest vertex
    function getDeepest(TreeCtx storage _tree)
        public
        view
        returns (uint32, uint32)
    {
        return (_tree.deepestVertex, _tree.deepestDepth);
    }

    function getRequiredDepths(uint32 _depth)
        private
        pure
        returns (uint32[] memory)
    {
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
