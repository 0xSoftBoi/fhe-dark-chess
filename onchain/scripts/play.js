// scripts/play.js
// LIVE Sepolia client demo for FogChessFHE using the Zama Relayer SDK.
//
//   node scripts/play.js
//
// This is the step that makes the trust real: encrypted inputs are produced with the
// relayer/SDK against the LIVE coprocessor, and the answer bit is revealed by the
// THRESHOLD KMS (no single key-holder), not the single mock key the hardhat tests use.
//
// Flow mirrors test/FogChessFHE.js, but seats ONE key on both sides:
//   1. encrypt a known 64-cell board: an enemy rook (code 4) on square 60 (e8).
//   2. commitBoard(handles, inputProof).
//   3. inCheck(opp=self, king=4 (e1), oppWhite=true) as a REAL tx — it mutates the
//      public `result` ebool and ACL-grants it to msg.sender.
//   4. read result() (bytes32 handle) and user-decrypt the ebool.
// Expected bit: TRUE — an open rook on the e-file checks the king on e1.
//
// NOTE: the demo signer plays BOTH seats (it commits the board AND is the `opp` whose
// board inCheck reads). In a real game these are two different players/keys; here a
// single funded key keeps the runbook to one account. opp = signer.address below.

const { ethers } = require("ethers");
// In Node you MUST import the /node subpath of the relayer SDK (native node-tfhe; no
// initSDK/WASM step). The /web and /bundle entry points are for browsers.
// (Uncertainty: this require path is per the shipped 0.4.3 package map; the SDK is not
//  installed in this repo, so it is verified from research, not run here.)
const { createInstance, SepoliaConfig } = require("@zama-fhe/relayer-sdk/node");

// Load .env if present (SEPOLIA_RPC_URL / PRIVATE_KEY / FOGCHESS_ADDRESS).
try {
  require("dotenv").config();
} catch (_) {
  /* dotenv optional; env can also be exported in the shell */
}

// --- required env (fail loudly + early, before any async work) ---
const { SEPOLIA_RPC_URL, PRIVATE_KEY, FOGCHESS_ADDRESS } = process.env;
for (const [k, v] of Object.entries({ SEPOLIA_RPC_URL, PRIVATE_KEY, FOGCHESS_ADDRESS })) {
  if (!v) {
    throw new Error(
      `Missing required env var ${k}. Copy .env.example to .env and fill it in ` +
        `(deploy first to get FOGCHESS_ADDRESS).`,
    );
  }
}

// commitBoard(bytes32[64], bytes); inCheck(address,uint8,bool) is a state-mutating tx;
// result() is the public ebool getter (bytes32 handle) that inCheck ACL-grants to caller.
const ABI = [
  "function commitBoard(bytes32[64] cells, bytes proof) external",
  "function inCheck(address opp, uint8 king, bool oppWhite) external returns (bytes32)",
  "function result() view returns (bytes32)",
];

async function main() {
  const provider = new ethers.JsonRpcProvider(SEPOLIA_RPC_URL);
  const signer = new ethers.Wallet(PRIVATE_KEY, provider);
  const userAddress = signer.address;
  const opp = userAddress; // single-key demo: signer plays both seats
  const contract = new ethers.Contract(FOGCHESS_ADDRESS, ABI, signer);

  console.log("Signer / opponent:", userAddress);
  console.log("Contract:", FOGCHESS_ADDRESS);

  // createInstance fetches FHE keys/CRS from the live relayer; SepoliaConfig has every
  // contract address baked in but NO `network` field, so we inject the RPC URL.
  const instance = await createInstance({ ...SepoliaConfig, network: SEPOLIA_RPC_URL });

  // --- 1. build the known encrypted board: rook (code 4) on square 60 (e8) ---
  const board = new Array(64).fill(0);
  board[60] = 4; // enemy rook on e8
  const input = instance.createEncryptedInput(FOGCHESS_ADDRESS, userAddress);
  for (let i = 0; i < 64; i++) input.add8(board[i]); // 64 handles, well within proof capacity
  const { handles, inputProof } = await input.encrypt();

  // --- 2. commit the board (ethers v6 accepts Uint8Array for bytes32[]/bytes) ---
  console.log("commitBoard...");
  await (await contract.commitBoard(handles, inputProof)).wait();

  // --- 3. inCheck as a REAL tx (mutates `result`, grants ACL to msg.sender) ---
  console.log("inCheck(opp, king=4, oppWhite=true)...");
  await (await contract.inCheck(opp, 4, true)).wait();

  // --- 4. read the granted ebool handle and user-decrypt it ---
  const resultHandle = await contract.result(); // bytes32
  const handleHex = ethers.hexlify(resultHandle); // userDecrypt result map is keyed by 0x-hex

  const keypair = instance.generateKeypair();
  const startTimestamp = Math.floor(Date.now() / 1000);
  const durationDays = 10;
  const contractAddresses = [FOGCHESS_ADDRESS];

  const eip712 = instance.createEIP712(
    keypair.publicKey,
    contractAddresses,
    startTimestamp,
    durationDays,
  );
  const signature = await signer.signTypedData(
    eip712.domain,
    { UserDecryptRequestVerification: eip712.types.UserDecryptRequestVerification },
    eip712.message,
  );

  const res = await instance.userDecrypt(
    [{ handle: handleHex, contractAddress: FOGCHESS_ADDRESS }],
    keypair.privateKey,
    keypair.publicKey,
    signature.replace(/^0x/, ""), // userDecrypt wants the signature WITHOUT the 0x prefix
    contractAddresses,
    userAddress,
    startTimestamp,
    durationDays,
  );

  const inCheck = res[handleHex]; // boolean for an ebool
  console.log("\ninCheck bit =", inCheck, "(expected: true — open rook on e-file checks e1)");
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
