# fhe-dark-chess

The other side of the wall [zk-dark-chess](https://github.com/0xSoftBoi/zk-dark-chess)
stopped at. ZK lets you prove a move is legal against **your own** committed board — but
fog-of-war chess also has predicates that depend on the **opponent's** hidden pieces, and
no proof against your own board can decide them:

- **capturing a hidden enemy piece** — is the square I'm moving to occupied?
- **a slider blocked by a hidden enemy** — is there an enemy piece on my rook's path?
- **check** — does any hidden enemy piece attack my king?

These are *joint predicates over two secret boards*. **FHE** can compute them: evaluate the
predicate homomorphically over the opponent's **encrypted** board and reveal only the
answer bit. This repo does exactly that, for real, with Zama's [`tfhe-rs`](https://github.com/zama-ai/tfhe-rs).

```
$ cargo run --release
1. occupancy(e8) = true  (captured piece code = 4)   [35ms]
2. blocked(a1->a4) = true  (pawn on a3 blocks)        [135ms]
3. in_check(e1) = true  (open enemy rook on e-file)   [7.6s]
[secret-king extrapolation] ... ~8 min (not run)
```

Each line leaks exactly one bit (or, for a capture, the one captured square) — the rest of
the opponent's board stays encrypted.

## The honest boundary: computation vs trust

A local demo encrypts **both** boards under a single `ClientKey`, so the key-holder is
exactly the referee we set out to remove. So be precise about what each layer buys:

- **`tfhe-rs` locally proves the COMPUTATION is referee-free** — the predicate runs on
  ciphertext and only the answer bit is ever decrypted. (This is what `cargo test` checks:
  the encrypted result matches the plaintext oracle on known positions.)
- **The TRUST (no single key-holder) needs a threshold KMS.** On Zama's
  [fhEVM](https://docs.zama.ai/fhevm) the board lives on-chain as ciphertext handles, a
  coprocessor runs the FHE math, and a threshold committee — none of whom can decrypt
  alone — reveals only the ACL-permitted bit. That's `contracts/FogChessFHE.sol`
  (a design sketch, not deployed: it needs the Zama network).

So: the maths is real and runs today; the trust distribution is designed, not deployed.

## Layout

- `src/lib.rs` — encrypted board + the predicates (`occupancy`, `blocked_by_opponent`,
  `in_check`) and their plaintext oracles.
- `src/main.rs` — the timed demo above.
- `tests/predicates.rs` — encrypted-result-vs-plaintext-truth on known positions.
- `contracts/FogChessFHE.sol` — the fhEVM on-chain design (euint8 board, the predicate via
  `FHE.*`, ACL + threshold decryption). **Sketch, not deployed.**

## Run

```bash
cargo test --release    # 4 tests: each predicate computed on ENCRYPTED boards == plaintext truth
cargo run  --release    # the timed demo (measured on an Apple Silicon laptop)
```

`check` at ~7.6s and a secret-king variant at ~8 min are the honest cost of FHE today —
fine for correspondence play, not yet for blitz. See [the write-up](https://0xsoftboi.github.io/blog/the-other-side-of-the-wall/)
and [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE). Educational; not audited.
