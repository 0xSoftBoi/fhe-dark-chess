// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {FHE, euint8, ebool, externalEuint8} from "@fhevm/solidity/lib/FHE.sol";
import {ZamaEthereumConfig} from "@fhevm/solidity/config/ZamaConfig.sol";

/// @title FogChessFHE
/// @notice The on-chain half of fhe-dark-chess: the board lives as on-chain ENCRYPTED
///         handles (`euint8` per cell), the joint fog-of-war predicates run on the
///         coprocessor, and only the one answer bit is ACL-gated for the caller to
///         decrypt. This is the real fhEVM SDK (compiles + runs in the mock coprocessor);
///         on a live deployment a threshold KMS — not any single key-holder — performs the
///         decryption. Mirrors the tfhe-rs reference in ../src/lib.rs.
///
/// Piece codes (each player's board holds only THEIR pieces; 0 = empty): 1=P 2=N 3=B 4=R 5=Q 6=K.
/// `inCheck` here considers only the opponent's pieces as blockers/attackers (your own
/// blockers you'd prune off-chain, since revealing them would break YOUR fog) — matching
/// the reference's `no_blockers` case.
contract FogChessFHE is ZamaEthereumConfig {
    uint8 internal constant P = 1;
    uint8 internal constant N = 2;
    uint8 internal constant B = 3;
    uint8 internal constant R = 4;
    uint8 internal constant Q = 5;
    uint8 internal constant K = 6;

    /// Each player's encrypted board.
    mapping(address => euint8[64]) private board;
    /// The most recent predicate result, ACL-granted to the caller that produced it.
    ebool public result;

    /// Seat: commit your 64-cell encrypted board (client packs all cells in ONE input).
    function commitBoard(externalEuint8[64] calldata cells, bytes calldata proof) external {
        for (uint256 i = 0; i < 64; i++) {
            euint8 c = FHE.fromExternal(cells[i], proof);
            board[msg.sender][i] = c;
            FHE.allowThis(c);
        }
    }

    /// occupancy(X): is the opponent's (public) square X occupied? -> one encrypted bit.
    function occupancy(address opp, uint8 x) external returns (ebool r) {
        r = FHE.ne(board[opp][x], FHE.asEuint8(0));
        result = _grant(r);
    }

    /// blockedSlider: is any opponent piece on the (public) path between two squares?
    function blockedSlider(address opp, uint8[] calldata path) external returns (ebool acc) {
        acc = FHE.asEbool(false);
        for (uint256 i = 0; i < path.length; i++) {
            acc = FHE.or(acc, FHE.ne(board[opp][path[i]], FHE.asEuint8(0)));
        }
        result = _grant(acc);
    }

    /// inCheck: does any opponent piece attack the (public) king square? -> one bit.
    function inCheck(address opp, uint8 king, bool oppWhite) external returns (ebool chk) {
        int256 kr = int256(uint256(king)) / 8;
        int256 kf = int256(uint256(king)) % 8;
        chk = FHE.asEbool(false);

        // sliding rays: 4 orthogonal (R/Q) + 4 diagonal (B/Q)
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(1), int256(0), R, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(-1), int256(0), R, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(0), int256(1), R, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(0), int256(-1), R, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(1), int256(1), B, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(1), int256(-1), B, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(-1), int256(1), B, Q));
        chk = FHE.or(chk, _ray(opp, kr, kf, int256(-1), int256(-1), B, Q));

        // knight offsets
        chk = FHE.or(chk, _off(opp, kr + 1, kf + 2, N));
        chk = FHE.or(chk, _off(opp, kr + 2, kf + 1, N));
        chk = FHE.or(chk, _off(opp, kr - 1, kf + 2, N));
        chk = FHE.or(chk, _off(opp, kr - 2, kf + 1, N));
        chk = FHE.or(chk, _off(opp, kr + 1, kf - 2, N));
        chk = FHE.or(chk, _off(opp, kr + 2, kf - 1, N));
        chk = FHE.or(chk, _off(opp, kr - 1, kf - 2, N));
        chk = FHE.or(chk, _off(opp, kr - 2, kf - 1, N));

        // pawn attackers (enemy pawns attack "forward" in their own direction)
        int256 pdr = oppWhite ? int256(-1) : int256(1);
        chk = FHE.or(chk, _off(opp, kr + pdr, kf - 1, P));
        chk = FHE.or(chk, _off(opp, kr + pdr, kf + 1, P));

        // adjacent enemy king
        for (int256 dr = -1; dr <= 1; dr++) {
            for (int256 df = -1; df <= 1; df++) {
                if (dr == 0 && df == 0) continue;
                chk = FHE.or(chk, _off(opp, kr + dr, kf + df, K));
            }
        }

        result = _grant(chk);
    }

    // ── helpers ───────────────────────────────────────────────────────────────────────

    function _grant(ebool v) internal returns (ebool) {
        FHE.allowThis(v);
        FHE.allow(v, msg.sender);
        return v;
    }

    /// OR of "an enemy slider of type a/b sits here", gated by a running "ray still clear".
    function _ray(address opp, int256 kr, int256 kf, int256 dr, int256 df, uint8 a, uint8 b)
        internal
        returns (ebool acc)
    {
        acc = FHE.asEbool(false);
        ebool clear = FHE.asEbool(true);
        int256 r = kr + dr;
        int256 f = kf + df;
        while (r >= 0 && r < 8 && f >= 0 && f < 8) {
            euint8 cell = board[opp][uint256(r * 8 + f)];
            ebool isAtk = FHE.or(FHE.eq(cell, FHE.asEuint8(a)), FHE.eq(cell, FHE.asEuint8(b)));
            acc = FHE.or(acc, FHE.and(clear, isAtk));
            clear = FHE.and(clear, FHE.eq(cell, FHE.asEuint8(0))); // opponent piece blocks further
            r += dr;
            f += df;
        }
    }

    /// "an enemy piece of `code` sits on (r,f)" (false if off-board).
    function _off(address opp, int256 r, int256 f, uint8 code) internal returns (ebool) {
        if (r >= 0 && r < 8 && f >= 0 && f < 8) {
            return FHE.eq(board[opp][uint256(r * 8 + f)], FHE.asEuint8(code));
        }
        return FHE.asEbool(false);
    }
}
