require("@fhevm/hardhat-plugin");
require("@nomicfoundation/hardhat-toolbox");
// Loads SEPOLIA_RPC_URL / PRIVATE_KEY from a local .env (see .env.example). Optional:
// if dotenv is absent or no .env exists this is a harmless no-op and the mock network
// (default `hardhat`, chainId 31337) still works without any env vars.
try {
  require("dotenv").config();
} catch (_) {
  /* dotenv not installed: only matters for the live `sepolia` network below */
}

module.exports = {
  solidity: {
    version: "0.8.27",
    settings: { optimizer: { enabled: true, runs: 800 }, evmVersion: "cancun" },
  },
  networks: {
    // Default in-memory mock fhEVM (mock coprocessor + single mock key) — UNCHANGED.
    hardhat: { chainId: 31337 },
    // Live Zama fhEVM on Ethereum Sepolia (real coprocessor + threshold KMS).
    // Config (ACL/Coprocessor/KMSVerifier) auto-resolves on-chain by chainid 11155111
    // via the inherited ZamaEthereumConfig — no per-network Solidity change needed.
    sepolia: {
      url: process.env.SEPOLIA_RPC_URL || "",
      chainId: 11155111,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
    },
  },
  mocha: { timeout: 600000 },
};
