import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import { solidity } from "ethereum-waffle";

import { TestTree } from "../src/types/TestTree";
import { TestTree__factory } from "../src/types/factories/TestTree__factory";

use(solidity);

describe("TestTree", async () => {
    let testTree: TestTree;
    const TreeId = 0;
    // const TestTreeString = ethers.utils.toUtf8Bytes("TestTree");
    // var TestTreeStringBytes = "0x"
    // for (var i = 0; i < TestTreeString.length; i++) {
    //     TestTreeStringBytes += TestTreeString[i].toString(16);
    // }

    beforeEach(async () => {
        await deployments.fixture();

        const [user] = await ethers.getSigners();

        const TreeAddress = (await deployments.get("Tree"))
            .address;
        const { deploy } = deployments;
        const { address } = await deploy("TestTree", {
            from: await user.getAddress(),
            log: true,
            libraries: {
                ["Tree"]: TreeAddress,
            },
        });

        testTree = TestTree__factory.connect(address, user);
    });

    it("insertVertex", async () => {
        await expect(
            testTree.insertVertex(8),
            "Insertion to invalid parent index"
        ).to.be.revertedWith("parent index exceeds current tree size");

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

        const vertex8 = await (testTree.getVertex(vertex8Index));

        expect(
            vertex8.ancestors,
            "Vertex8 ancestors should match"
        ).to.deep.equal(vertex8Ancestors);

        // vertex9
        await expect(
            testTree.insertVertex(vertex7Index),
            "Insert vertex9 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(TreeId, vertex7Index);

        const vertex9 = await (testTree.getVertex(vertex9Index));

        expect(
            vertex9.ancestors,
            "Vertex9 ancestors should match"
        ).to.deep.equal(vertex9Ancestors);

        // requirement
        await expect(
            testTree.getVertex(vertex9Index + 1),
        ).to.be.revertedWith("vertex index exceeds current tree size");
    });

    it("getAncestorAtDepth", async () => {
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
        ).to.be.revertedWith("vertex index exceeds current tree size");

        await expect(
            testTree.getAncestorAtDepth(vertex9Index, vertex9Depth + 1),
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

        // deepest
        expect(
            await testTree.getDeepest(),
            "Deepest vertex should match"
        ).to.deep.equal([vertex7Index, vertex7Depth]);

        // depth
        expect(
            await testTree.getDepth(vertex7Index),
            "Depth of deepest vertex should match"
        ).to.equal(vertex7Depth);
    });
});
