import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, getNamedAccounts } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();
    const { TreeLibrary } = await deployments.all();

    await deploy("TestTree", {
        from: deployer,
        log: true,
        libraries: {
            ["TreeLibrary"]: TreeLibrary.address
        },
    });
};

export default func;
export const tags = ['TestTree'];
