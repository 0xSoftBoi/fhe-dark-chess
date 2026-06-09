require("@fhevm/hardhat-plugin");
require("@nomicfoundation/hardhat-toolbox");
module.exports = {
  solidity: {
    version: "0.8.27",
    settings: { optimizer: { enabled: true, runs: 800 }, evmVersion: "cancun" },
  },
  networks: { hardhat: { chainId: 31337 } },
  mocha: { timeout: 600000 },
};
