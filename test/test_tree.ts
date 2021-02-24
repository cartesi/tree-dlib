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

        const address = (await deployments.get("TestTree")).address;
        testTree = TestTree__factory.connect(address, user);
    });

    it("insertVertex", async () => {
        await expect(
            testTree.insertVertex(8),
            "Insertion to invalid parent index"
        ).to.be.revertedWith("parent index exceeds current tree size");

        const vertex7Depth = 7;
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
    });

    it("getAncestorAtDepth", async () => {
        const vertex7Depth = 7;
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
    });
});
