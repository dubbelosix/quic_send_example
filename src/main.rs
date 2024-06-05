use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use solana_client::connection_cache::ConnectionCache;
use bincode;
use solana_client::tpu_connection::TpuConnection;
use solana_sdk::signature::{keypair, Signer};
use solana_client::rpc_client;
use solana_client::rpc_config::RpcGetVoteAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::Transaction;

const MAINNET_RPC: &'static str = "https://api.mainnet-beta.solana.com";
pub fn transfer_001(rpc_url: &str, keypair_path: &str, receiver: &str) -> Transaction {
    let signer = keypair::read_keypair_file(keypair_path).unwrap();
    let receiver = Pubkey::from_str(receiver).unwrap();
    let target_instruction = transfer(&signer.pubkey(), &receiver, 10_000_000);
    let client = rpc_client::RpcClient::new(rpc_url);
    let latest_block_hash = client.get_latest_blockhash().unwrap();
    Transaction::new_signed_with_payer(&[target_instruction], Some(&signer.pubkey()), &[&signer], latest_block_hash)
}

pub fn get_validator_quic_info(rpc_url: &str) -> HashMap<Pubkey,(u64, SocketAddr)> {
    let rpc_client = rpc_client::RpcClient::new(rpc_url);

    let vote_accounts = rpc_client.get_vote_accounts_with_config(RpcGetVoteAccountsConfig::default()).unwrap();
    let mut activated_stake: HashMap<Pubkey, u64> = HashMap::default();
    for vote_account in vote_accounts.current {
        activated_stake.insert(vote_account.node_pubkey.parse().unwrap(),vote_account.activated_stake );
    }

    let mut nodes = HashMap::default();
    for contact_info in rpc_client.get_cluster_nodes().unwrap() {
        let pubkey = Pubkey::from_str(&contact_info.pubkey).unwrap();
        if let Some(stake) = activated_stake.get(&pubkey) {
            if let Some(contact_info) =  contact_info.tpu_quic {
                nodes.insert(
                    pubkey,
                    (*stake, contact_info)
                );
            }
        }
    };
    nodes
}

fn get_connection_cache(keypair_path: &str, addr: &str) -> ConnectionCache {
    let keypair = keypair::read_keypair_file(keypair_path).unwrap();
    let ipaddr = IpAddr::V4(Ipv4Addr::from_str(&addr).unwrap());
    ConnectionCache::new_with_client_options(
        "test-connection",
        2,
        None,
        Some((&keypair, ipaddr)),
        None
    )
}

fn main() {
    let connection_cache = get_connection_cache("identity.json", "0.0.0.0");
    let validator_quic_info = get_validator_quic_info(MAINNET_RPC);
    let galaxy = validator_quic_info.get(&Pubkey::from_str("DtdSSG8ZJRZVv5Jx7K1MeWp7Zxcu19GD5wQRGRpQ9uMF").unwrap()).unwrap();
    let conn = connection_cache.get_connection(&galaxy.1);
    println!("{:?}",conn.server_addr());
    let txn = transfer_001(MAINNET_RPC, "identity.json", "HbvJJaRJu77dzH7KveoPiro8QUwjWS55RwCMB24cLtMT");
    conn.send_data(&bincode::serialize(&txn).unwrap()).unwrap();
}
