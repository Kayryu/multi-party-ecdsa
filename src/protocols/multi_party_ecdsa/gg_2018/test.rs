#![allow(non_snake_case)]

/*
    Multi-party ECDSA

    Copyright 2018 by Kzen Networks

    This file is part of Multi-party ECDSA library
    (https://github.com/KZen-networks/multi-party-ecdsa)

    Multi-party ECDSA is free software: you can redistribute
    it and/or modify it under the terms of the GNU General Public
    License as published by the Free Software Foundation, either
    version 3 of the License, or (at your option) any later version.

    @license GPL-3.0+ <https://github.com/KZen-networks/multi-party-ecdsa/blob/master/LICENSE>
*/

use crate::protocols::multi_party_ecdsa::gg_2018::party_i::{
    KeyGenBroadcastMessage1, KeyGenDecommitMessage1, Keys
};

use crate::protocols::multi_party_ecdsa::gg_2018::mock::{keygen_t_n_parties, sign, double_sign};

#[test]
fn test_keygen_t1_n2() {
    keygen_t_n_parties(1, 2);
}

#[test]
fn test_keygen_t2_n3() {
    keygen_t_n_parties(2, 3);
}

#[test]
fn test_keygen_t2_n4() {
    keygen_t_n_parties(2, 4);
}

#[test]
fn test_sign_n5_t2_ttag4() {
    sign(2, 5, 5, vec![0, 1, 2, 3, 4])
}

#[test]
fn test_sign_n8_t4_ttag6() {
    sign(4, 8, 6, vec![0, 1, 2, 4, 6, 7])
}

#[test]
fn test_sign_n20_t16_ttag17() {
    sign(16, 20, 17, vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 18, 19])
}

#[test]
fn test_double_sign() {
    double_sign(2, 5, 5, vec![0, 1, 2, 3, 4])
}

#[test]
fn test_serialize_deserialize() {
    use serde_json;

    let k = Keys::create(0);
    let (commit, decommit) = k.phase1_broadcast_phase3_proof_of_correct_key();

    let encoded = serde_json::to_string(&commit).unwrap();
    let decoded: KeyGenBroadcastMessage1 = serde_json::from_str(&encoded).unwrap();
    assert_eq!(commit.com, decoded.com);

    let encoded = serde_json::to_string(&decommit).unwrap();
    let decoded: KeyGenDecommitMessage1 = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decommit.y_i, decoded.y_i);
}
