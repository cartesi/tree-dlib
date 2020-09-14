import {
    BuidlerRuntimeEnvironment,
    DeployFunction
} from "@nomiclabs/buidler/types";

const func: DeployFunction = async (bre: BuidlerRuntimeEnvironment) => {
    const { deployments, getNamedAccounts } = bre;
    const { deploy } = deployments;
    const a = await getNamedAccounts();
    const { deployer } = await getNamedAccounts();
    const Tree = await deployments.get("TreeLibrary");

    await deploy("TestTree", {
        from: deployer,
        log: true,
        libraries: {
            ["TreeLibrary"]: Tree.address
        },
    });
};

export default func;
export const tags = ['TestTree'];
