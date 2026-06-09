# Deploy FogChessFHE to live Zama fhEVM on Ethereum Sepolia

> ✅ **Done once already.** `FogChessFHE` is live on Sepolia at
> [`0x99db76240c884F35133A6c9a12249C67a906da12`](https://sepolia.etherscan.io/address/0x99db76240c884F35133A6c9a12249C67a906da12).
> A board was committed encrypted, `inCheck` ran on the live coprocessor, and the threshold
> KMS decrypted the result bit (`true`, open rook on the e-file). This runbook reproduces it.

The hardhat tests run `FogChessFHE` in the **mock coprocessor**, which uses a single mock
key — it proves the *computation* is referee-free but not the *trust*. This runbook takes
the same contract to the **live** Sepolia fhEVM, where the trust becomes real:

- a **coprocessor** runs the FHE math (`FHE.ne`, `FHE.or`, `FHE.select`, comparisons) over
  the on-chain encrypted board handles — it never sees plaintext; and
- a **threshold KMS committee** holds the decryption key in shares, so **no single party
  can decrypt**. Only the one ACL-permitted answer bit (`result`) is revealed, and only to
  the address `inCheck` granted it to.

That replacement of "one mock key" with "a threshold committee + a real coprocessor" is the
whole point — this is the step that makes the no-single-referee claim true rather than
designed.

## Prerequisites

- **Node 20+** and npm.
- A **Sepolia private key with testnet ETH**. fhEVM ops cost substantially more gas than
  plaintext EVM (every FHE op calls the coprocessor/ACL), and the contract is large — fund
  generously. Faucets:
  - Google Cloud Web3 faucet: https://cloud.google.com/application/web3/faucet/ethereum/sepolia
  - Alchemy: https://sepoliafaucet.com
  - (Infura / QuickNode faucets also work.)
- A **Sepolia RPC URL** (Infura/Alchemy/QuickNode, or a public endpoint).

## 1. Install

```bash
cd onchain
npm install
```

This pulls in `@zama-fhe/relayer-sdk@0.4.3` (the client SDK) and `dotenv`.

## 2. Configure `.env`

```bash
cp .env.example .env
# edit .env:
#   SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/<key>   (or your provider)
#   PRIVATE_KEY=0x<funded sepolia key>
#   FOGCHESS_ADDRESS=                                     (leave blank for now)
```

`.env` is git-ignored — never commit real keys.

## 3. Compile + deploy

```bash
npx hardhat compile
npx hardhat run scripts/deploy.js --network sepolia
```

`FogChessFHE` has no constructor args; the live ACL / Coprocessor / KMSVerifier addresses
auto-resolve on-chain by `block.chainid == 11155111` via the inherited `ZamaEthereumConfig`.
The script prints:

```
FogChessFHE deployed at: 0x....
```

## 4. Set the address, then play

Put the printed address into `.env`:

```
FOGCHESS_ADDRESS=0x....
```

Then run the live client demo:

```bash
node scripts/play.js
```

`scripts/play.js` (Relayer SDK 0.4.3):

1. encrypts a known 64-cell board (an enemy rook, code 4, on square 60 / e8) off-chain
   against the live relayer and commits it with `commitBoard(handles, inputProof)`;
2. calls `inCheck(opp, king=4 (e1), oppWhite=true)` as a real tx, which runs the
   fog-of-war check predicate on the coprocessor and ACL-grants the result bit to the
   caller;
3. reads the granted `result()` handle and **user-decrypts** it through the threshold KMS.

Expected output:

```
inCheck bit = true (expected: true — open rook on e-file checks e1)
```

The demo signer plays **both** seats (it commits the board and is the `opp` whose board is
read) so the runbook needs only one funded key; in a real game these are two different
players with two keys.

## Notes / gotchas

- On live Sepolia, encrypted inputs MUST be produced with the relayer/SDK and decryptions
  are **asynchronous** via the Gateway/KMS — they do not resolve synchronously like in the
  mock. The mock-only hardhat tests (`fhevm.userDecryptEbool`) will not run against Sepolia.
- If a tx reverts out-of-gas, raise the gas limit rather than assuming a logic bug — fhEVM
  ops are expensive.
- `createInstance` fetches FHE keys/CRS from `relayer.testnet.zama.org` at startup; a
  failure there surfaces as an instance-creation error, not an encryption error.
