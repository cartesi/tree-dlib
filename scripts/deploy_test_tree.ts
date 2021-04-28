import hre from "hardhat";
import fs from "fs";
const {deployments, ethers, getNamedAccounts} = hre;

async function main() {
    const { Tree } = await deployments.all()
    const TestTreeFactory = await ethers.getContractFactory(
        "TestTree",
        {
            libraries: {
                Tree: Tree.address
            }
        },
    );
    const testTree = await TestTreeFactory.deploy();

    // Write address in 'TestTree.address' .
    fs.writeFileSync('TestTree.address', testTree.address);
  }

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
