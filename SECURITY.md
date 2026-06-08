# Security notes

Educational; **not audited**; a research prototype.

## What is real here, and what is designed

- **Real (runs in `cargo test`/`cargo run`):** the three joint predicates are computed
  homomorphically over the opponent's **encrypted** board with `tfhe-rs`, and only the
  answer bit (or, for a capture, the one captured square) is decrypted. The encrypted
  result is checked against a plaintext oracle on known positions. This establishes that
  the **computation** leaks nothing beyond the intended bit.
- **Designed, not deployed (`contracts/FogChessFHE.sol`):** the threshold-KMS trust model
  that removes the single key-holder. It needs the Zama fhEVM network (coprocessor +
  threshold committee + Gateway). Not run here.

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
