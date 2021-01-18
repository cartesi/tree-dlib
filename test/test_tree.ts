import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import { solidity } from "ethereum-waffle";

import { TestTree } from "../src/types/TestTree";
import { TestTree__factory } from "../src/types/factories/TestTree__factory";

use(solidity);

describe("TestTree", async () => {
    let testTree: TestTree;

    beforeEach(async () => {
        await deployments.fixture();

        const [user] = await ethers.getSigners();

        const address = (await deployments.get("TestTree")).address;
        testTree = TestTree__factory.connect(address, user);
    });

    it("initial tree", async () => {
        const treeSize = await testTree.getTreeSize();

        for (let i = 0; i < treeSize; ++i) {
            expect(
                ethers.utils.toUtf8String((await testTree.getVertex(i)).data),
                "Vertex data doesn't match"
            ).to.equal(`Vertex ${i}`);
        }
    });

    it("insertVertex", async () => {
        await expect(
            testTree.insertVertex(8, ethers.utils.toUtf8Bytes("Invalid insertion")),
            "Insertion to invalid parent index"
        ).to.be.revertedWith("parent index exceeds current tree size");

        const vertex8Data = ethers.utils.toUtf8Bytes("Vertex 8");
        const vertex9Data = ethers.utils.toUtf8Bytes("Vertex 9");
        const vertex7Depth = 7;
        const vertex7Index = 7;
        const vertex8Index = 8;
        const vertex9Index = 9;
        const vertex8Ancestors = [7, 6, 4, 0];
        const vertex9Ancestors = vertex8Ancestors;
        var vertex8DataBytes = "0x"
        for (var i = 0; i < vertex8Data.length; i++) {
            vertex8DataBytes += vertex8Data[i].toString(16);
        }
        var vertex9DataBytes = "0x"
        for (var i = 0; i < vertex9Data.length; i++) {
            vertex9DataBytes += vertex9Data[i].toString(16);
        }

        // vertex8
        await expect(
            testTree.insertVertex(vertex7Index, vertex8Data),
            "Insert vertex8 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(vertex8Index, vertex7Index, vertex7Depth + 1, vertex8DataBytes);

        const vertex8 = await (testTree.getVertex(vertex8Index));

        expect(
            vertex8.ancestors,
            "Vertex8 ancestors should match"
        ).to.deep.equal(vertex8Ancestors);

        // vertex9
        await expect(
            testTree.insertVertex(vertex7Index, vertex9Data),
            "Insert vertex9 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(vertex9Index, vertex7Index, vertex7Depth + 1, vertex9DataBytes);

        const vertex9 = await (testTree.getVertex(vertex9Index));

        expect(
            vertex9.ancestors,
            "Vertex9 ancestors should match"
        ).to.deep.equal(vertex9Ancestors);
    });

    it("getAncestorAtDepth", async () => {
        const vertex8Data = ethers.utils.toUtf8Bytes("Vertex 8");
        const vertex9Data = ethers.utils.toUtf8Bytes("Vertex 9");
        const vertex7Depth = 7;
        const vertex7Index = 7;
        const vertex8Index = 8;
        const vertex9Index = 9;
        const vertex8Depth = 8;
        const vertex9Depth = 8;
        var vertex8DataBytes = "0x"
        for (var i = 0; i < vertex8Data.length; i++) {
            vertex8DataBytes += vertex8Data[i].toString(16);
        }
        var vertex9DataBytes = "0x"
        for (var i = 0; i < vertex9Data.length; i++) {
            vertex9DataBytes += vertex9Data[i].toString(16);
        }

        // vertex8
        await expect(
            testTree.insertVertex(vertex7Index, vertex8Data),
            "Insert vertex8 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(vertex8Index, vertex7Index, vertex7Depth + 1, vertex8DataBytes);

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
            testTree.insertVertex(vertex7Index, vertex9Data),
            "Insert vertex9 should emit event"
        )
            .to.emit(testTree, "VertexInserted")
            .withArgs(vertex9Index, vertex7Index, vertex7Depth + 1, vertex9DataBytes);

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
