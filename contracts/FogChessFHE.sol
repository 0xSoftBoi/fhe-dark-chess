// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

// DESIGN SKETCH — NOT DEPLOYED. This is the on-chain half of fhe-dark-chess: where the
// *trust* becomes real. The Rust crate proves the COMPUTATION is referee-free (the
// predicate runs homomorphically and leaks only the answer bit) but encrypts both boards
// under one key. fhEVM removes that single key: the board lives on-chain as ciphertext
// handles, the Zama coprocessor runs the FHE math, and a THRESHOLD KMS committee — no
// single member of which can decrypt — reveals only the one bit the ACL permits.
//
// This file uses the Zama FHE Solidity API for shape; it is illustrative, not compiled
// against a pinned fhevm release here. See https://docs.zama.ai/fhevm.

import {FHE, euint8, ebool} from "@fhevm/solidity/lib/FHE.sol";

contract FogChessFHE {
    // Each player's 64-cell board as encrypted handles (0 = empty, 1..6 = P,N,B,R,Q,K).
    // Only ciphertext handles are on-chain; the plaintext never is.
    mapping(address => euint8[64]) private board;

    // Piece codes as trivially-encrypted constants for comparison.
    function _code(uint8 v) internal returns (euint8) { return FHE.asEuint8(v); }

    /// Seat: store your encrypted board. Inputs are external ciphertexts + their proof.
    function commitBoard(externalEuint8[64] calldata cells, bytes calldata proof) external {
        for (uint256 i = 0; i < 64; i++) {
            euint8 c = FHE.fromExternal(cells[i], proof);
            board[msg.sender][i] = c;
            FHE.allowThis(c);            // this contract may compute on it
        }
    }

    /// occupancy(X): is the (public) destination square X occupied by the opponent's piece?
    /// Returns an encrypted bit; the caller requests a threshold decryption of just this bit.
    function occupancy(address opponent, uint256 x) external returns (ebool occupied) {
        occupied = FHE.ne(board[opponent][x], _code(0));
        FHE.allow(occupied, msg.sender);     // ACL: only the mover may decrypt the answer
        // Off-chain: FHE.requestDecryption / the Gateway routes this to the threshold KMS,
        // which returns ONLY this bit — never the rest of the opponent's board.
    }

    /// in_check(king): does any opponent piece attack the public king square? OR-reduces
    /// ray attackers (gated by a running "clear" flag) + knight/pawn/king offsets to one
    /// ebool — the joint predicate ZK-against-your-own-board cannot decide. (Mirrors
    /// src/lib.rs::in_check; ray geometry elided for brevity.)
    function inCheck(address opponent, uint256 king, uint8 oppColor)
        external
        returns (ebool check)
    {
        euint8[64] storage opp = board[opponent];
        check = FHE.asEbool(false);
        // for each (square s, attackerTypeA, attackerTypeB) along the rays/offsets:
        //   ebool isAtk = FHE.or(FHE.eq(opp[s], _code(A)), FHE.eq(opp[s], _code(B)));
        //   check       = FHE.or(check, FHE.and(clear, isAtk));
        //   clear       = FHE.and(clear, FHE.eq(opp[s], _code(0)));
        // (see the Rust reference for the exact ray walk; ~250 FHE ops total)
        FHE.allow(check, msg.sender);
    }
}
