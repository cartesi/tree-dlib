import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, getNamedAccounts } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();
    const { Tree } = await deployments.all();

    await deploy("TestTree", {
        from: deployer,
        log: true,
        libraries: {
            ["Tree"]: Tree.address
        },
    });
};

func.tags = ['TestTree'];
func.dependencies=['Tree'];
export default func;
