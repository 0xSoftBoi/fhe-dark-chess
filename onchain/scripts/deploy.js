// scripts/deploy.js
// Deploy FogChessFHE to a network (intended: Sepolia).
//   npx hardhat run scripts/deploy.js --network sepolia
//
// FogChessFHE has no constructor args (it only inherits ZamaEthereumConfig, whose
// constructor reads the live ACL/Coprocessor/KMSVerifier addresses by block.chainid).
const { ethers, network } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  if (!deployer) {
    throw new Error(
      "No signer available. Set PRIVATE_KEY in .env (and SEPOLIA_RPC_URL) before deploying to sepolia.",
    );
  }
  const net = await ethers.provider.getNetwork();
  console.log("Network:", network.name, "chainId:", net.chainId.toString());
  console.log("Deployer:", deployer.address);

  const factory = await ethers.getContractFactory("FogChessFHE");
  const contract = await factory.deploy(); // no constructor args
  await contract.waitForDeployment(); // ethers v6

  const addr = await contract.getAddress();
  console.log("FogChessFHE deployed at:", addr);
  console.log("\nNext: set FOGCHESS_ADDRESS in your .env, then run `node scripts/play.js`.");
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
