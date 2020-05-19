use crate::protocols::multi_party_ecdsa::gg_2018::party_i::{verify, Keys, LocalSignature, Parameters, PartyPrivate, Phase5ADecom1, Phase5Com1, SharedKeys, SignKeys, SignatureRecid};
use crate::utilities::mta::{MessageA, MessageB};

use curv::arithmetic::traits::Converter;
use curv::cryptographic_primitives::hashing::hash_sha256::HSha256;
use curv::cryptographic_primitives::hashing::traits::Hash;
use curv::cryptographic_primitives::proofs::sigma_dlog::DLogProof;
use curv::cryptographic_primitives::secret_sharing::feldman_vss::VerifiableSS;
use curv::elliptic::curves::traits::*;
use curv::{FE, GE};
use paillier::*;

pub fn keygen_t_n_parties(t: u16, n: u16) -> (Vec<Keys>, Vec<SharedKeys>, Vec<GE>, GE, VerifiableSS, FE) {
    let parames = Parameters {
        threshold: t,
        share_count: n,
    };
    let (t, n) = (t as usize, n as usize);

    // select u
    let party_keys_vec = (0..n).map(Keys::create).collect::<Vec<Keys>>();

    // compute committed
    let (bc1_vec, decom_vec): (Vec<_>, Vec<_>) = party_keys_vec
        .iter()
        .map(|k| k.phase1_broadcast_phase3_proof_of_correct_key())
        .unzip();

    // public key of the partial private key.
    let y_vec = (0..n).map(|i| decom_vec[i].y_i).collect::<Vec<GE>>();
    let mut y_vec_iter = y_vec.iter();
    let head = y_vec_iter.next().unwrap();
    let tail = y_vec_iter;
    // get real public key by sum.
    let y_sum = tail.fold(head.clone(), |acc, x| acc + x);

    let mut vss_scheme_vec = Vec::new();
    let mut secret_shares_vec = Vec::new();
    let mut index_vec = Vec::new();

    // create Feldman-VSS
    let vss_result: Vec<_> = party_keys_vec
        .iter()
        .map(|k| {
            k.phase1_verify_com_phase3_verify_correct_key_phase2_distribute(
                &parames, &decom_vec, &bc1_vec,
            )
                .expect("invalid key")
        })
        .collect();

    for (vss_scheme, secret_shares, index) in vss_result {
        vss_scheme_vec.push(vss_scheme);
        secret_shares_vec.push(secret_shares); // cannot unzip
        index_vec.push(index);
    }

    let vss_scheme_for_test = vss_scheme_vec.clone();

    // Transpose matrix
    let party_shares = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    let vec_j = &secret_shares_vec[j];
                    vec_j[i]
                })
                .collect::<Vec<FE>>()
        })
        .collect::<Vec<Vec<FE>>>();

    let mut shared_keys_vec = Vec::new();
    let mut dlog_proof_vec = Vec::new();
    for (i, key) in party_keys_vec.iter().enumerate() {
        let (shared_keys, dlog_proof) = key
            .phase2_verify_vss_construct_keypair_phase3_pok_dlog(
                &parames,
                &y_vec,
                &party_shares[i],
                &vss_scheme_vec,
                &index_vec[i] + 1,
            )
            .expect("invalid vss");
        shared_keys_vec.push(shared_keys);
        dlog_proof_vec.push(dlog_proof);
    }

    let pk_vec = (0..n).map(|i| dlog_proof_vec[i].pk).collect::<Vec<GE>>();

    //both parties run:
    Keys::verify_dlog_proofs(&parames, &dlog_proof_vec, &y_vec).expect("bad dlog proof");

    //test
    let xi_vec = (0..=t).map(|i| shared_keys_vec[i].x_i).collect::<Vec<FE>>();
    let x = vss_scheme_for_test[0]
        .clone()
        .reconstruct(&index_vec[0..=t], &xi_vec);
    let sum_u_i = party_keys_vec.iter().fold(FE::zero(), |acc, x| acc + x.u_i);

    // why equal?  sum_u_i is the sum of each party private key.
    // x is calc by lagrange interpolation at zero.
    assert_eq!(x, sum_u_i);

    (
        party_keys_vec,
        shared_keys_vec,
        pk_vec,
        y_sum,
        vss_scheme_for_test[0].clone(),
        x,
    )
}

pub fn sign(t: u16, n: u16, ttag: u16, s: Vec<usize>) {
    // full key gen emulation
    // party_keys_vec: party private key
    // shared_keys_vec: private key of vss
    // _pk_vec : public key of vss
    // y:  real public key
    // vss_scheme: why single?
    // x: real private key
    let (party_keys_vec, shared_keys_vec, _pk_vec, y, vss_scheme, x) = keygen_t_n_parties(t, n);

    // make sure that we have t<t'<n and the group s contains id's for t' parties
    // TODO: make sure s has unique id's and they are all in range 0..n
    // TODO: make sure this code can run when id's are not in ascending order
    assert!(ttag > t);
    let ttag = ttag as usize;
    assert_eq!(s.len(), ttag);

    let message: [u8; 4] = [79, 77, 69, 82];
    let message_bn = HSha256::create_hash(&[&BigInt::from(&message[..])]);

    let (sig, local_sig_vec) = sign_impl(ttag, party_keys_vec, shared_keys_vec, y, vss_scheme, s, message_bn);

    check_sig(&sig.r, &sig.s, &local_sig_vec[0].m, &y);
    check_normal_sig(&x, &sig.r, &sig.s, &local_sig_vec[0].m, &y);
}

pub fn double_sign(t: u16, n: u16, ttag: u16, s: Vec<usize>) {
    let (party_keys_vec, shared_keys_vec, _pk_vec, y, vss_scheme, x) = keygen_t_n_parties(t, n);

    // make sure that we have t<t'<n and the group s contains id's for t' parties
    // TODO: make sure s has unique id's and they are all in range 0..n
    // TODO: make sure this code can run when id's are not in ascending order
    assert!(ttag > t);
    let ttag = ttag as usize;
    assert_eq!(s.len(), ttag);

    let message: [u8; 4] = [79, 77, 69, 82];
    let message_bn = HSha256::create_hash(&[&BigInt::from(&message[..])]);

    let (sig, local_sig_vec) = sign_impl(ttag, party_keys_vec.clone(), shared_keys_vec.clone(), y, vss_scheme.clone(), s.clone(), message_bn.clone());

    check_sig(&sig.r, &sig.s, &local_sig_vec[0].m, &y);

    let (sig2, local_sig_vec2) = sign_impl(ttag, party_keys_vec, shared_keys_vec, y, vss_scheme, s, message_bn);

    check_sig(&sig2.r, &sig2.s, &local_sig_vec2[0].m, &y);

    println!("sig1 : {:?} \n sig2: {:?}", sig, sig2);
}

fn sign_impl(ttag: usize, party_keys_vec: Vec<Keys>, shared_keys_vec: Vec<SharedKeys>, y: GE, vss_scheme: VerifiableSS, s: Vec<usize>, message_bn: BigInt) -> (SignatureRecid, Vec<LocalSignature>) {
    let private_vec = (0..shared_keys_vec.len())
        .map(|i| PartyPrivate::set_private(party_keys_vec[i].clone(), shared_keys_vec[i].clone()))
        .collect::<Vec<PartyPrivate>>();

    // each party creates a signing key. This happens in parallel IRL. In this test we
    // create a vector of signing keys, one for each party.
    // throughout i will index parties
    let sign_keys_vec = (0..ttag)
        .map(|i| SignKeys::create(&private_vec[s[i]], &vss_scheme, s[i], &s))
        .collect::<Vec<SignKeys>>();

    // each party computes [Ci,Di] = com(g^gamma_i) and broadcast the commitments
    // a hash commitment scheme
    let (bc1_vec, decommit_vec1): (Vec<_>, Vec<_>) =
        sign_keys_vec.iter().map(|k| k.phase1_broadcast()).unzip();

    // each party i sends encryption of k_i under her Paillier key
    // m_a_vec = [ma_0;ma_1;,...]
    let m_a_vec: Vec<_> = sign_keys_vec
        .iter()
        .enumerate()
        .map(|(i, k)| MessageA::a(&k.k_i, &party_keys_vec[s[i]].ek).0)
        .collect();

    // each party i sends responses to m_a_vec she received (one response with input gamma_i and one with w_i)
    // m_b_gamma_vec_all is a matrix where column i is a vector of message_b's that party i answers to all ma_{j!=i} using paillier key of party j to answer to ma_j

    // aggregation of the n messages of all parties
    let mut m_b_gamma_vec_all = Vec::new();
    let mut beta_vec_all = Vec::new();
    let mut m_b_w_vec_all = Vec::new();
    let mut ni_vec_all = Vec::new();

    for (i, key) in sign_keys_vec.iter().enumerate() {
        let mut m_b_gamma_vec = Vec::new();
        let mut beta_vec = Vec::new();
        let mut m_b_w_vec = Vec::new();
        let mut ni_vec = Vec::new();

        for j in 0..ttag - 1 {
            let ind = if j < i { j } else { j + 1 };

            let (m_b_gamma, beta_gamma, _) = MessageB::b(
                &key.gamma_i,  // g_gamma_i has commitment
                &party_keys_vec[s[ind]].ek,
                m_a_vec[ind].clone(),
            );
            let (m_b_w, beta_wi, _) =
                MessageB::b(&key.w_i, &party_keys_vec[s[ind]].ek, m_a_vec[ind].clone());

            m_b_gamma_vec.push(m_b_gamma);
            beta_vec.push(beta_gamma);
            m_b_w_vec.push(m_b_w);
            ni_vec.push(beta_wi);
        }
        m_b_gamma_vec_all.push(m_b_gamma_vec.clone());
        beta_vec_all.push(beta_vec.clone());
        m_b_w_vec_all.push(m_b_w_vec.clone());
        ni_vec_all.push(ni_vec.clone());
    }

    // Here we complete the MwA protocols by taking the mb matrices and starting with the first column generating the appropriate message
    // for example for index i=0 j=0 we need party at index s[1] to answer to mb that party s[0] sent, completing a protocol between s[0] and s[1].
    //  for index i=1 j=0 we need party at index s[0] to answer to mb that party s[1]. etc.
    // IRL each party i should get only the mb messages that other parties sent in response to the party i ma's.
    // TODO: simulate as IRL
    let mut alpha_vec_all = Vec::new();
    let mut miu_vec_all = Vec::new();

    for i in 0..ttag {
        let mut alpha_vec = Vec::new();
        let mut miu_vec = Vec::new();

        let m_b_gamma_vec_i = &m_b_gamma_vec_all[i];
        let m_b_w_vec_i = &m_b_w_vec_all[i];

        for j in 0..ttag - 1 {
            let ind = if j < i { j } else { j + 1 };
            let m_b = m_b_gamma_vec_i[j].clone();

            let alpha_ij_gamma = m_b
                .verify_proofs_get_alpha(&party_keys_vec[s[ind]].dk, &sign_keys_vec[ind].k_i)
                .expect("wrong dlog or m_b");
            let m_b = m_b_w_vec_i[j].clone();
            let alpha_ij_wi = m_b
                .verify_proofs_get_alpha(&party_keys_vec[s[ind]].dk, &sign_keys_vec[ind].k_i)
                .expect("wrong dlog or m_b");

            // since we actually run two MtAwc each party needs to make sure that the values B are the same as the public values
            // here for b=w_i the parties already know W_i = g^w_i  for each party so this check is done here. for b = gamma_i the check will be later when g^gamma_i will become public
            // currently we take the W_i from the other parties signing keys
            // TODO: use pk_vec (first change from x_i to w_i) for this check.
            assert_eq!(m_b.b_proof.pk, sign_keys_vec[i].g_w_i);

            alpha_vec.push(alpha_ij_gamma);
            miu_vec.push(alpha_ij_wi);
        }
        alpha_vec_all.push(alpha_vec.clone());
        miu_vec_all.push(miu_vec.clone());
    }

    let mut delta_vec = Vec::new();
    let mut sigma_vec = Vec::new();

    for i in 0..ttag {
        let delta = sign_keys_vec[i].phase2_delta_i(&alpha_vec_all[i], &beta_vec_all[i]);
        let sigma = sign_keys_vec[i].phase2_sigma_i(&miu_vec_all[i], &ni_vec_all[i]);
        delta_vec.push(delta);
        sigma_vec.push(sigma);
    }

    // all parties broadcast delta_i and compute delta_i ^(-1)
    let delta_inv = SignKeys::phase3_reconstruct_delta(&delta_vec);

    // de-commit to g^gamma_i from phase1, test comm correctness, and that it is the same value used in MtA.
    // Return R

    let _g_gamma_i_vec = (0..ttag)
        .map(|i| sign_keys_vec[i].g_gamma_i)
        .collect::<Vec<GE>>();

    let R_vec = (0..ttag)
        .map(|_| {
            // each party i tests all B = g^b = g ^ gamma_i she received.
            let b_proof_vec = (0..ttag)
                .map(|j| {
                    let b_gamma_vec = &m_b_gamma_vec_all[j];
                    &b_gamma_vec[0].b_proof
                })
                .collect::<Vec<&DLogProof>>();
            SignKeys::phase4(&delta_inv, &b_proof_vec, decommit_vec1.clone(), &bc1_vec)
                .expect("bad gamma_i decommit")
        })
        .collect::<Vec<GE>>();

    let mut local_sig_vec = Vec::new();

    // each party computes s_i but don't send it yet. we start with phase5
    for i in 0..ttag {
        let local_sig = LocalSignature::phase5_local_sig(
            &sign_keys_vec[i].k_i,
            &message_bn,
            &R_vec[i],
            &sigma_vec[i],
            &y,
        );
        local_sig_vec.push(local_sig);
    }

    let mut phase5_com_vec: Vec<Phase5Com1> = Vec::new();
    let mut phase_5a_decom_vec: Vec<Phase5ADecom1> = Vec::new();
    let mut helgamal_proof_vec = Vec::new();
    let mut dlog_proof_rho_vec = Vec::new();
    // we notice that the proof for V= R^sg^l, B = A^l is a general form of homomorphic elgamal.
    for sig in &local_sig_vec {
        let (phase5_com, phase_5a_decom, helgamal_proof, dlog_proof_rho) =
            sig.phase5a_broadcast_5b_zkproof();
        phase5_com_vec.push(phase5_com);
        phase_5a_decom_vec.push(phase_5a_decom);
        helgamal_proof_vec.push(helgamal_proof);
        dlog_proof_rho_vec.push(dlog_proof_rho);
    }

    let mut phase5_com2_vec = Vec::new();
    let mut phase_5d_decom2_vec = Vec::new();
    for i in 0..ttag {
        let mut phase_5a_decom_vec_clone = phase_5a_decom_vec.clone();
        let mut phase_5a_com_vec_clone = phase5_com_vec.clone();
        let mut phase_5b_elgamal_vec_clone = helgamal_proof_vec.clone();

        let _decom_i = phase_5a_decom_vec_clone.remove(i);
        let _com_i = phase_5a_com_vec_clone.remove(i);
        let _elgamal_i = phase_5b_elgamal_vec_clone.remove(i);
        //        for j in 0..s_minus_i.len() {
        let (phase5_com2, phase_5d_decom2) = local_sig_vec[i]
            .phase5c(
                &phase_5a_decom_vec_clone,
                &phase_5a_com_vec_clone,
                &phase_5b_elgamal_vec_clone,
                &dlog_proof_rho_vec,
                &phase_5a_decom_vec[i].V_i,
                &R_vec[0],
            )
            .expect("error phase5");
        phase5_com2_vec.push(phase5_com2);
        phase_5d_decom2_vec.push(phase_5d_decom2);
        //        }
    }

    // assuming phase5 checks passes each party sends s_i and compute sum_i{s_i}
    let mut s_vec: Vec<FE> = Vec::new();
    for sig in &local_sig_vec {
        let s_i = sig
            .phase5d(&phase_5d_decom2_vec, &phase5_com2_vec, &phase_5a_decom_vec)
            .expect("bad com 5d");
        s_vec.push(s_i);
    }

    // here we compute the signature only of party i=0 to demonstrate correctness.
    s_vec.remove(0);
    let sig = local_sig_vec[0]
        .output_signature(&s_vec)
        .expect("verification failed");

    assert_eq!(local_sig_vec[0].y, y);
    verify(&sig, &local_sig_vec[0].y, &local_sig_vec[0].m).unwrap();

    (sig, local_sig_vec)
}

fn check_normal_sig(x: &FE, r: &FE, s: &FE, msg: &BigInt, pk: &GE) {
    use secp256k1::{verify, Message, SecretKey, PublicKey, PublicKeyFormat, Signature, sign};

    let mut privkey_a = [0u8; 32];
    for i in 0..32 {
        privkey_a[i] = x.get_element()[i];
    }

    let raw_msg = BigInt::to_vec(&msg);
    let mut msg: Vec<u8> = Vec::new(); // padding
    msg.extend(vec![0u8; 32 - raw_msg.len()]);
    msg.extend(raw_msg.iter());

    let ctx_privkey = SecretKey::parse(&privkey_a).unwrap();
    let ctx_message = Message::parse_slice(&raw_msg.as_slice()).unwrap();
    let ctx_public = PublicKey::from_secret_key(&ctx_privkey);
    println!("message: {:?} \n private key: {:?} \n public key: {:?}", ctx_message, ctx_privkey, ctx_public);

    // check public key
    let slice = pk.pk_to_key_slice();
    let mut raw_pk = Vec::new();
    if slice.len() != 65 {
        // after curv's pk_to_key_slice return 65 bytes, this can be removed
        raw_pk.insert(0, 4u8);
        raw_pk.extend(vec![0u8; 64 - slice.len()]);
        raw_pk.extend(slice);
    } else {
        raw_pk.extend(slice);
    }
    assert_eq!(raw_pk.len(), 65);
    let pk = PublicKey::parse_slice(&raw_pk, Some(PublicKeyFormat::Full)).unwrap();
    assert_eq!(pk, ctx_public);
    println!("public check: \n 1:{:?} \n 2:{:?}", pk, ctx_public);

    let (signature, id) = sign(&ctx_message, &ctx_privkey);
    let reconstructed = Signature::parse_der(signature.serialize_der().as_ref()).unwrap();
    let is_correct = verify(&ctx_message, &signature, &ctx_public);
    println!("recover id: {:?}", id);
    assert!(is_correct);

    // convert tss signature to normal signature
    let mut compact: Vec<u8> = Vec::new();
    let bytes_r = &r.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_r.len()]);
    compact.extend(bytes_r.iter());

    let bytes_s = &s.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_s.len()]);
    compact.extend(bytes_s.iter());

    let secp_sig = Signature::parse_slice(compact.as_slice()).unwrap();
    let is_correct = verify(&ctx_message, &secp_sig, &pk);
    assert!(is_correct);

    // recover???
    assert_eq!(secp_sig.r, reconstructed.r);
    // assert_eq!(secp_sig.s, reconstructed.s);

}

pub fn check_sig(r: &FE, s: &FE, msg: &BigInt, pk: &GE) {
    use secp256k1::{verify, Message, PublicKey, PublicKeyFormat, Signature};

    let raw_msg = BigInt::to_vec(&msg);
    let mut msg: Vec<u8> = Vec::new(); // padding
    msg.extend(vec![0u8; 32 - raw_msg.len()]);
    msg.extend(raw_msg.iter());

    let msg = Message::parse_slice(msg.as_slice()).unwrap();
    let slice = pk.pk_to_key_slice();
    let mut raw_pk = Vec::new();
    if slice.len() != 65 {
        // after curv's pk_to_key_slice return 65 bytes, this can be removed
        raw_pk.insert(0, 4u8);
        raw_pk.extend(vec![0u8; 64 - slice.len()]);
        raw_pk.extend(slice);
    } else {
        raw_pk.extend(slice);
    }

    assert_eq!(raw_pk.len(), 65);

    let pk = PublicKey::parse_slice(&raw_pk, Some(PublicKeyFormat::Full)).unwrap();

    let mut compact: Vec<u8> = Vec::new();
    let bytes_r = &r.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_r.len()]);
    compact.extend(bytes_r.iter());

    let bytes_s = &s.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_s.len()]);
    compact.extend(bytes_s.iter());

    let secp_sig = Signature::parse_slice(compact.as_slice()).unwrap();

    let is_correct = verify(&msg, &secp_sig, &pk);
    assert!(is_correct);
}