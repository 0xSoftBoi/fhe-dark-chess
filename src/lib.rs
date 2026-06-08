//! fhe-dark-chess: the joint fog-of-war predicates that ZK-against-your-own-board can't
//! decide — computed homomorphically over the OPPONENT's encrypted board, revealing only
//! the answer bit.
//!
//! Piece codes (each player's board holds only THEIR pieces; 0 = empty):
//!   1=P 2=N 3=B 4=R 5=Q 6=K
//!
//! Honesty: a single ClientKey encrypts everything here, so the key-holder is a referee.
//! That makes the local demo prove the COMPUTATION is referee-free (only the bit leaks),
//! not the trust. Distributing the key across a threshold committee (Zama KMS) is the
//! production piece — see contracts/FogChessFHE.sol and the README.

use tfhe::prelude::*;
use tfhe::{ClientKey, FheBool, FheUint8};

pub const P: u8 = 1;
pub const N: u8 = 2;
pub const B: u8 = 3;
pub const R: u8 = 4;
pub const Q: u8 = 5;
pub const K: u8 = 6;

/// An encrypted 64-cell board (one player's pieces).
pub struct EncBoard(pub Vec<FheUint8>);

impl EncBoard {
    pub fn encrypt(board: &[u8; 64], ck: &ClientKey) -> Self {
        EncBoard(board.iter().map(|&c| FheUint8::encrypt(c, ck)).collect())
    }
}

#[inline]
fn rf(sq: usize) -> (i32, i32) {
    ((sq / 8) as i32, (sq % 8) as i32)
}
#[inline]
fn sq(r: i32, f: i32) -> Option<usize> {
    if (0..8).contains(&r) && (0..8).contains(&f) {
        Some((r * 8 + f) as usize)
    } else {
        None
    }
}

/// Squares strictly between `from` and `to` along a straight/diagonal line (empty if not aligned).
pub fn ray_between(from: usize, to: usize) -> Vec<usize> {
    let (fr, ff) = rf(from);
    let (tr, tf) = rf(to);
    let dr = (tr - fr).signum();
    let df = (tf - ff).signum();
    let straight = (fr == tr) || (ff == tf);
    let diagonal = (tr - fr).abs() == (tf - ff).abs();
    if !(straight || diagonal) {
        return vec![];
    }
    let mut out = vec![];
    let (mut r, mut f) = (fr + dr, ff + df);
    while (r, f) != (tr, tf) {
        match sq(r, f) {
            Some(s) => out.push(s),
            None => break,
        }
        r += dr;
        f += df;
    }
    out
}

/// occupancy(X): is the (public) destination square X occupied by an enemy piece?
/// The single homomorphic primitive that resolves a capture in fog of war.
pub fn occupancy(opp: &EncBoard, x: usize) -> FheBool {
    opp.0[x].ne(0u8)
}

/// blocked_slider: is any opponent piece sitting on the path between from and to?
/// (Your own blockers you already know in the clear — this only asks about hidden ones.)
pub fn blocked_by_opponent(opp: &EncBoard, from: usize, to: usize) -> FheBool {
    let path = ray_between(from, to);
    let mut acc = FheBool::encrypt_trivial(false);
    for s in path {
        acc = acc | opp.0[s].ne(0u8);
    }
    acc
}

/// in_check: does any of the opponent's hidden pieces attack the (public) king square?
/// OR-reduces, over the encrypted opponent board, attacks along the 8 rays (gated by a
/// running "still clear" flag), plus knight / pawn / king-adjacency offsets.
///
/// `my_blockers[s]` true means one of YOUR pieces sits on s (known in the clear): it
/// truncates a ray, so the FHE loop only extends to your own blockers.
/// `opp_is_white` is the opponent's colour (sets pawn-attack direction).
pub fn in_check(
    opp: &EncBoard,
    king: usize,
    my_blockers: &[bool; 64],
    opp_is_white: bool,
) -> FheBool {
    let (kr, kf) = rf(king);
    let mut acc = FheBool::encrypt_trivial(false);

    // sliding rays: 4 orthogonal (rook/queen) + 4 diagonal (bishop/queen)
    let dirs: [(i32, i32, bool); 8] = [
        (1, 0, true), (-1, 0, true), (0, 1, true), (0, -1, true), // orthogonal
        (1, 1, false), (1, -1, false), (-1, 1, false), (-1, -1, false), // diagonal
    ];
    for (dr, df, orthogonal) in dirs {
        let (slider_a, slider_b) = if orthogonal { (R, Q) } else { (B, Q) };
        let mut clear = FheBool::encrypt_trivial(true);
        let (mut r, mut f) = (kr + dr, kf + df);
        while let Some(s) = sq(r, f) {
            if my_blockers[s] {
                break; // your own piece blocks the ray — known in the clear
            }
            let is_attacker = opp.0[s].eq(slider_a) | opp.0[s].eq(slider_b);
            acc = acc | (clear.clone() & is_attacker);
            clear = clear & opp.0[s].eq(0u8); // an opponent piece blocks further out
            r += dr;
            f += df;
        }
    }

    // knight attackers
    for (dr, df) in [(1, 2), (2, 1), (-1, 2), (-2, 1), (1, -2), (2, -1), (-1, -2), (-2, -1)] {
        if let Some(s) = sq(kr + dr, kf + df) {
            if !my_blockers[s] {
                acc = acc | opp.0[s].eq(N);
            }
        }
    }

    // pawn attackers: an enemy pawn attacks "forward" in its own direction.
    // white pawns attack toward higher ranks, so a white pawn on (kr-1, kf±1) hits the king.
    let pdr = if opp_is_white { -1 } else { 1 };
    for df in [-1, 1] {
        if let Some(s) = sq(kr + pdr, kf + df) {
            if !my_blockers[s] {
                acc = acc | opp.0[s].eq(P);
            }
        }
    }

    // adjacent enemy king
    for dr in -1..=1 {
        for df in -1..=1 {
            if dr == 0 && df == 0 {
                continue;
            }
            if let Some(s) = sq(kr + dr, kf + df) {
                if !my_blockers[s] {
                    acc = acc | opp.0[s].eq(K);
                }
            }
        }
    }

    acc
}

// ── plaintext oracles (the ground truth the FHE result must match) ────────────────────

pub fn plain_blocked(opp: &[u8; 64], from: usize, to: usize) -> bool {
    ray_between(from, to).iter().any(|&s| opp[s] != 0)
}

pub fn plain_in_check(opp: &[u8; 64], king: usize, my_blockers: &[bool; 64], opp_is_white: bool) -> bool {
    let (kr, kf) = rf(king);
    let dirs: [(i32, i32, bool); 8] = [
        (1, 0, true), (-1, 0, true), (0, 1, true), (0, -1, true),
        (1, 1, false), (1, -1, false), (-1, 1, false), (-1, -1, false),
    ];
    for (dr, df, orth) in dirs {
        let (sa, sb) = if orth { (R, Q) } else { (B, Q) };
        let (mut r, mut f) = (kr + dr, kf + df);
        while let Some(s) = sq(r, f) {
            if my_blockers[s] { break; }
            if opp[s] == sa || opp[s] == sb { return true; }
            if opp[s] != 0 { break; }
            r += dr; f += df;
        }
    }
    for (dr, df) in [(1, 2), (2, 1), (-1, 2), (-2, 1), (1, -2), (2, -1), (-1, -2), (-2, -1)] {
        if let Some(s) = sq(kr + dr, kf + df) {
            if !my_blockers[s] && opp[s] == N { return true; }
        }
    }
    let pdr = if opp_is_white { -1 } else { 1 };
    for df in [-1, 1] {
        if let Some(s) = sq(kr + pdr, kf + df) {
            if !my_blockers[s] && opp[s] == P { return true; }
        }
    }
    for dr in -1..=1 {
        for df in -1..=1 {
            if dr == 0 && df == 0 { continue; }
            if let Some(s) = sq(kr + dr, kf + df) {
                if !my_blockers[s] && opp[s] == K { return true; }
            }
        }
    }
    false
}
