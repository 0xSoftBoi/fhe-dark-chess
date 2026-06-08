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

fn setup() -> tfhe::ClientKey {
    let (ck, sk) = generate_keys(ConfigBuilder::default().build());
    set_server_key(sk);
    ck
}

#[test]
fn occupancy_and_capture_reveal_only_X() {
    let ck = setup();
    let opp = board(&[(28, R), (40, P)]); // enemy rook on 28, pawn on 40
    let enc = EncBoard::encrypt(&opp, &ck);

    // occupied square -> true; empty square -> false (one bit each)
    assert!(occupancy(&enc, 28).decrypt(&ck));
    assert!(!occupancy(&enc, 20).decrypt(&ck));

    // capture resolution reveals ONLY the captured square's contents
    let captured: u8 = enc.0[28].decrypt(&ck);
    assert_eq!(captured, R);
}

#[test]
fn blocked_slider_matches_plaintext() {
    let ck = setup();
    // clear file a1->a4: no opponent piece between
    let clear = board(&[(63, K)]);
    let enc_clear = EncBoard::encrypt(&clear, &ck);
    assert_eq!(blocked_by_opponent(&enc_clear, 0, 24).decrypt(&ck), plain_blocked(&clear, 0, 24));
    assert!(!blocked_by_opponent(&enc_clear, 0, 24).decrypt(&ck));

    // opponent pawn on a3 (square 16) blocks a1->a4
    let blocked = board(&[(16, P)]);
    let enc_b = EncBoard::encrypt(&blocked, &ck);
    assert_eq!(blocked_by_opponent(&enc_b, 0, 24).decrypt(&ck), plain_blocked(&blocked, 0, 24));
    assert!(blocked_by_opponent(&enc_b, 0, 24).decrypt(&ck));
}

#[test]
fn check_via_rook_and_blocker() {
    let ck = setup();
    let king = 4usize; // e1
    let no_blk = [false; 64];

    // enemy rook on e8 (60), clear file -> CHECK
    let opp = board(&[(60, R)]);
    let enc = EncBoard::encrypt(&opp, &ck);
    let fhe = in_check(&enc, king, &no_blk, true).decrypt(&ck);
    assert_eq!(fhe, plain_in_check(&opp, king, &no_blk, true));
    assert!(fhe, "open rook on the file must be check");

    // same, but an enemy pawn on e5 (36) blocks the rook -> NO check
    let opp2 = board(&[(60, R), (36, P)]);
    let enc2 = EncBoard::encrypt(&opp2, &ck);
    let fhe2 = in_check(&enc2, king, &no_blk, true).decrypt(&ck);
    assert_eq!(fhe2, plain_in_check(&opp2, king, &no_blk, true));
    assert!(!fhe2, "blocked rook is not check");
}

#[test]
fn check_via_knight_and_quiet_position() {
    let ck = setup();
    let king = 4usize; // e1 (rank 0, file 4)
    let no_blk = [false; 64];

    // enemy knight on d3 / f3 style square that attacks e1: (dr,df)=(2,1) -> r2 f5 = 21
    let opp = board(&[(21, N)]);
    let enc = EncBoard::encrypt(&opp, &ck);
    let fhe = in_check(&enc, king, &no_blk, true).decrypt(&ck);
    assert_eq!(fhe, plain_in_check(&opp, king, &no_blk, true));
    assert!(fhe, "knight on 21 attacks e1");

    // a far-away rook on a different file/rank with no line -> NO check
    let quiet = board(&[(63, R), (2, N)]); // h8 rook (not aligned), knight on c1 (no offset)
    let encq = EncBoard::encrypt(&quiet, &ck);
    let fheq = in_check(&encq, king, &no_blk, true).decrypt(&ck);
    assert_eq!(fheq, plain_in_check(&quiet, king, &no_blk, true));
    assert!(!fheq, "quiet position is not check");
}
