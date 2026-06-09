# Security notes

Educational; **not audited**; a research prototype.

## What is real here, and what is designed

- **Real (runs in `cargo test`/`cargo run`):** the three joint predicates are computed
  homomorphically over the opponent's **encrypted** board with `tfhe-rs`, and only the
  answer bit (or, for a capture, the one captured square) is decrypted. The encrypted
  result is checked against a plaintext oracle on known positions. This establishes that
  the **computation** leaks nothing beyond the intended bit.
- **Real SDK, mock coprocessor (`onchain/`):** `FogChessFHE.sol` compiles against
  `@fhevm/solidity` 0.11.1 and **runs in the hardhat mock coprocessor** (`npx hardhat test`)
  — an encrypted board is committed on-chain, `occupancy`/`inCheck` run on the coprocessor,
  and the caller decrypts one ACL-gated bit. This exercises the real fhEVM API + flow.
- **Deployed + verified on live Sepolia:** `FogChessFHE` is live at
  `0x99db76240c884F35133A6c9a12249C67a906da12`. An encrypted board was committed, `inCheck`
  ran on the **live coprocessor**, and the result bit was decrypted by the **real threshold
  KMS** (no single key-holder) — `true` for an open rook on the e-file. The single-mock-key
  caveat no longer applies; the trust is real.
- **Residual caveats:** the live run used a single demo signer for both seats (a real game
  needs two keys); it is testnet only; and FHE ops are gas/HCU-heavy and slower than the
  mock. See `onchain/DEPLOY.md` to reproduce.

## The single-key caveat (read this)

The local demo encrypts **both** players' boards under one `ClientKey`. Whoever holds it
can decrypt everything — i.e. it is the referee this project exists to remove. The demo is
therefore a proof that the *computation* is referee-free, **not** that the *trust* is. A
production deployment MUST hold the decryption key in a threshold committee (Zama KMS), so
no single party — including the chain operator — can read a board. Treat any claim of
"trustless fog of war" from the local binary alone as false.

## Other limitations

- **Cost.** Measured on an Apple Silicon laptop: occupancy ~35 ms, blocked-slider ~135 ms,
  public-king check ~7.6 s; the **secret-king** variant multiplexes over 64 squares to
  ~8 minutes (extrapolated, not run). Workable for correspondence play; not interactive.
  On fhEVM the coprocessor (not your laptop) runs this, with its own latency.
- **Encoding trust.** Each player encrypts their own board; nothing here forces that board
  to be a *legal* chess position. A complete protocol pairs this with a ZK proof (à la
  zk-dark-chess) that the committed/encrypted board is well-formed, plus input-ciphertext
  proofs on fhEVM.
- **Scope.** This is the predicate primitive, not a full game — no turn loop, move
  application, or win conditions. The `in_check` ray walk handles your-own-piece blockers
  in the clear (you know your board) and the opponent's blockers under FHE.

## Reporting

No deployment, no funds. Open an issue for a correctness bug (a predicate whose encrypted
result disagrees with the plaintext oracle).
