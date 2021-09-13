import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import { solidity } from "ethereum-waffle";

import { TestTree, TestTree__factory } from "../src/types";
import { getState } from "./getState";

use(solidity);

describe("TestTree", async () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let initialState: string;

    let testTree: TestTree;
    const TreeId = 0;

    beforeEach(async () => {
        await deployments.fixture();

        const [user] = await ethers.getSigners();

        const TreeAddress = (await deployments.get("Tree")).address;
        const { deploy } = deployments;
        const { address } = await deploy("TestTree", {
            from: await user.getAddress(),
            log: true,
            libraries: {
                ["Tree"]: TreeAddress,
            },
        });

        testTree = TestTree__factory.connect(address, user);

        if (enableDelegate) {
            initialState = JSON.stringify({
                tree_address: address,
                pos_instance: "0x0",
            });
        }
    });

    it("test insertVertex", async () => {
        await expect(
            testTree.insertVertex(8),
            "insertVertex should revert if parent index is invalid"
        ).to.be.revertedWith("parent index exceeds current tree size");

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                Object.keys(state.vertices).length,
                "Tree should remain size 8"
            ).to.equal(8);
        }

        const vertex7Index = 7;
        const vertex8Index = 8;
        const vertex9Index = 9;
        const vertex8Ancestors = [7, 6, 4, 0];
        const vertex9Ancestors = vertex8Ancestors;

        // vertex8
        await expect(
            testTree.insertVertex(vertex7Index),
            "Insert vertex8 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(TreeId, vertex7Index);

        const vertex8 = await testTree.getVertex(vertex8Index);

        expect(
            vertex8.ancestors,
            "Vertex8 ancestors should match"
        ).to.deep.equal(vertex8Ancestors);

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                Object.keys(state.vertices).length,
                "Tree should become size 9"
            ).to.equal(9);

            expect(
                state.vertices["8"],
                "Tree should include new vertex"
            ).to.deep.equal({
                depth: vertex8Index,
                index: vertex8Index,
                parent: vertex7Index,
            });
        }

        // vertex9
        await expect(
            testTree.insertVertex(vertex7Index),
            "Insert vertex9 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(TreeId, vertex7Index);

        const vertex9 = await testTree.getVertex(vertex9Index);

        expect(
            vertex9.ancestors,
            "Vertex9 ancestors should match"
        ).to.deep.equal(vertex9Ancestors);

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                Object.keys(state.vertices).length,
                "Tree should become size 10"
            ).to.equal(10);

            expect(
                state.vertices["9"],
                "Tree should include new vertex"
            ).to.deep.equal({
                depth: vertex8Index,
                index: vertex9Index,
                parent: vertex7Index,
            });
        }

        // requirement
        await expect(
            testTree.getVertex(vertex9Index + 1),
            "getVertex should revert if vertex index is invalid"
        ).to.be.revertedWith("vertex index exceeds current tree size");

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                state.vertices["10"],
                "Tree should include valid vertices only"
            ).to.be.undefined;
        }
    });

    it("test ancestors greater than 255", async () => {
        for (var i = 0; i < 258; ++i) {
            await testTree.insertVertex(i + 7);
        }

        const vertex256Index = 256;
        const vertex256Ancestors = [255, 254, 252, 248, 240, 224, 192, 128, 0];

        const vertex256 = await testTree.getVertex(vertex256Index);

        expect(
            vertex256.ancestors,
            "Verte256 ancestors should match"
        ).to.deep.equal(vertex256Ancestors);

        const vertex264Index = 264;
        const vertex264Ancestors = [263, 262, 260, 256];

        const vertex264 = await testTree.getVertex(vertex264Index);

        expect(
            vertex264.ancestors,
            "Vertex264 ancestors should match"
        ).to.deep.equal(vertex264Ancestors);
    });

    it("test getAncestorAtDepth", async () => {
        const vertex7Index = 7;
        const vertex8Index = 8;
        const vertex9Index = 9;
        const vertex8Depth = 8;
        const vertex9Depth = 8;

        // vertex8
        await expect(
            testTree.insertVertex(vertex7Index),
            "Insert vertex8 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(TreeId, vertex7Index);

        expect(
            await testTree.getAncestorAtDepth(vertex8Index, vertex8Depth),
            "Ancestor at depth should match"
        ).to.equal(vertex8Index);

        expect(
            await testTree.getAncestorAtDepth(vertex8Index, 0),
            "Ancestor at depth should match"
        ).to.equal(0);

        // vertex9
        await expect(
            testTree.insertVertex(vertex7Index),
            "Insert vertex9 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(TreeId, vertex7Index);

        expect(
            await testTree.getAncestorAtDepth(vertex9Index, vertex9Depth),
            "Ancestor at depth should match"
        ).to.equal(vertex9Index);

        expect(
            await testTree.getAncestorAtDepth(vertex9Index, 0),
            "Ancestor at depth should match"
        ).to.equal(0);

        // requirements
        await expect(
            testTree.getAncestorAtDepth(vertex9Index + 1, 0),
            "getAncestorAtDepth should revert if vertex index is invalid"
        ).to.be.revertedWith("vertex index exceeds current tree size");

        await expect(
            testTree.getAncestorAtDepth(vertex9Index, vertex9Depth + 1),
            "getAncestorAtDepth should revert if target deoth is invalid"
        ).to.be.revertedWith("search depth deeper than vertex depth");

        // branch
        for (const i of [1, 2, 3, 4, 5, 6, 7]) {
            expect(
                await testTree.getAncestorAtDepth(vertex9Index, i),
                "Ancestor at depth should match"
            ).to.equal(i);
        }
    });

    it("getter functions", async () => {
        const initialTreeSize = 8;
        const vertex7Index = 7;
        const vertex7Depth = 7;

        // tree size
        expect(
            await testTree.getTreeSize(),
            "Initial tree size should match"
        ).to.equal(initialTreeSize);

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                Object.keys(state.vertices).length,
                "Tree should remain initial size"
            ).to.equal(initialTreeSize);
        }

        // deepest
        expect(
            await testTree.getDeepest(),
            "Deepest vertex should match"
        ).to.deep.equal([vertex7Index, vertex7Depth]);

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                state.deepest[state.deepest.length - 1],
                "Tree deepest vertex should match"
            ).to.deep.equal({ depth: vertex7Depth, index: vertex7Index });
        }

        // depth
        expect(
            await testTree.getDepth(vertex7Index),
            "Depth of deepest vertex should match"
        ).to.equal(vertex7Depth);

        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));

            expect(
                state.vertices["7"].depth,
                "Depth of vertex should match"
            ).to.equal(vertex7Depth);
        }
    });
});
