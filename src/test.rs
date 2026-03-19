#![cfg(test)]

use super::*;
use soroban_sdk::token::TokenClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Bytes, BytesN, Env};
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}

fn hash_receiver(env: &Env, index: u32, address: Address, amount: i128) -> BytesN<32> {
    let receiver = Receiver {
        index,
        address,
        amount,
    };
    env.crypto().sha256(&receiver.to_xdr(env)).into()
}

fn hash_pair(env: &Env, lhs: &BytesN<32>, rhs: &BytesN<32>) -> BytesN<32> {
    let a = lhs.to_array();
    let b = rhs.to_array();
    let (left, right) = if a < b { (a, b) } else { (b, a) };

    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(&left);
    combined[32..].copy_from_slice(&right);
    env.crypto().sha256(&Bytes::from_slice(env, &combined)).into()
}

#[test]
fn test_valid_claim() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    token_admin_client.mint(&token_admin_client.address, &1000);

    let a0 = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    let l0 = hash_receiver(&env, 0, a0.clone(), 10);
    let l1 = hash_receiver(&env, 1, a1.clone(), 20);
    let l2 = hash_receiver(&env, 2, a2.clone(), 100);
    let l3 = hash_receiver(&env, 3, a3.clone(), 30);
    let h01 = hash_pair(&env, &l0, &l1);
    let h23 = hash_pair(&env, &l2, &l3);
    let root = hash_pair(&env, &h01, &h23);

    let contract_id = env.register(
        MerkleDistributionContract,
        MerkleDistributionContractArgs::__constructor(
            &root,
            &token.address,
            &1000,
            &token_admin_client.address,
        ),
    );
    let client = MerkleDistributionContractClient::new(&env, &contract_id);

    let proof = vec![&env, l3.clone(), h01.clone()];
    client.claim(&2_u32, &a2, &100_i128, &proof);

    assert_eq!(token.balance(&a2), 100);
    assert_eq!(token.balance(&contract_id), 900);
    assert!(env.auths().is_empty());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_double_claim() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    token_admin_client.mint(&token_admin_client.address, &1000);

    let a0 = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    let l0 = hash_receiver(&env, 0, a0.clone(), 10);
    let l1 = hash_receiver(&env, 1, a1.clone(), 20);
    let l2 = hash_receiver(&env, 2, a2.clone(), 100);
    let l3 = hash_receiver(&env, 3, a3.clone(), 30);
    let h01 = hash_pair(&env, &l0, &l1);
    let h23 = hash_pair(&env, &l2, &l3);
    let root = hash_pair(&env, &h01, &h23);

    let contract_id = env.register(
        MerkleDistributionContract,
        MerkleDistributionContractArgs::__constructor(
            &root,
            &token.address,
            &1000,
            &token_admin_client.address,
        ),
    );
    let client = MerkleDistributionContractClient::new(&env, &contract_id);

    let proof = vec![&env, l3.clone(), h01.clone()];
    client.claim(&2_u32, &a2, &100_i128, &proof);
    client.claim(&2_u32, &a2, &100_i128, &proof);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_bad_claim() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    token_admin_client.mint(&token_admin_client.address, &1000);

    let a0 = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    let l0 = hash_receiver(&env, 0, a0.clone(), 10);
    let l1 = hash_receiver(&env, 1, a1.clone(), 20);
    let l2 = hash_receiver(&env, 2, a2.clone(), 100);
    let l3 = hash_receiver(&env, 3, a3.clone(), 30);
    let h01 = hash_pair(&env, &l0, &l1);
    let h23 = hash_pair(&env, &l2, &l3);
    let root = hash_pair(&env, &h01, &h23);

    let contract_id = env.register(
        MerkleDistributionContract,
        MerkleDistributionContractArgs::__constructor(
            &root,
            &token.address,
            &1000,
            &token_admin_client.address,
        ),
    );
    let client = MerkleDistributionContractClient::new(&env, &contract_id);

    let proof = vec![&env, l3.clone(), h01.clone()];
    client.claim(&2_u32, &a2, &999_i128, &proof);
}
