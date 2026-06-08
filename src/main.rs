//! Timed demo: resolve the fog-of-war JOINT predicates over an encrypted opponent board.
//! Everything the opponent's hidden pieces touch — captures, blocked sliders, check —
//! decided homomorphically, revealing only the answer bit.

use std::time::Instant;
use fhe_dark_chess::*;
use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, ConfigBuilder};

fn board(cells: &[(usize, u8)]) -> [u8; 64] {
    let mut b = [0u8; 64];
    for &(s, p) in cells {
        b[s] = p;
    }
    b
}

fn main() {
    println!("== fhe-dark-chess: joint fog-of-war predicates over an ENCRYPTED board ==\n");
    let t = Instant::now();
    let (ck, sk) = generate_keys(ConfigBuilder::default().build());
    set_server_key(sk);
    println!("[setup] key generation: {:?} (one-time)\n", t.elapsed());

    // The opponent's hidden board: a rook on e8, a knight on f6, a pawn on a3.
    let opp = board(&[(60, R), (45, N), (16, P)]);
    let enc = EncBoard::encrypt(&opp, &ck);
    let king = 4usize; // our king on e1 (public)
    let no_blk = [false; 64];

    // 1. capture / occupancy of a public destination square
    let t = Instant::now();
    let occ = occupancy(&enc, 60).decrypt(&ck);
    let dt1 = t.elapsed();
    let captured: u8 = enc.0[60].decrypt(&ck); // reveals ONLY square 60
    println!("1. occupancy(e8) = {occ}  (captured piece code = {captured})   [{dt1:?}]");

    // 2. is a slider from a1 -> a4 blocked by a hidden enemy piece?
    let t = Instant::now();
    let blk = blocked_by_opponent(&enc, 0, 24).decrypt(&ck);
    let dt2 = t.elapsed();
    println!("2. blocked(a1->a4) = {blk}  (pawn on a3 blocks)   [{dt2:?}]");

    // 3. is our king in check from a hidden piece?  (the predicate ZK can't decide)
    let t = Instant::now();
    let chk = in_check(&enc, king, &no_blk, true).decrypt(&ck);
    let dt3 = t.elapsed();
    println!("3. in_check(e1) = {chk}  (open enemy rook on e-file)   [{dt3:?}]");

    // honesty: cost of the SECRET-king variant (king square hidden too).
    // Hiding the king square means running the predicate for every candidate square and
    // multiplexing the result -> ~64x the public-king cost.
    println!(
        "\n[secret-king extrapolation] public-king check took {dt3:?}; hiding the king square\n\
         multiplexes over 64 candidates -> ~{:?} (minutes-scale, not run here).",
        dt3 * 64
    );

    println!(
        "\nEvery line above leaked exactly one bit (or, for the capture, one square).\n\
         All boards here are under ONE key -> this proves the COMPUTATION is referee-free,\n\
         not the trust. Distributing the key is the threshold-KMS job (contracts/FogChessFHE.sol)."
    );
}
