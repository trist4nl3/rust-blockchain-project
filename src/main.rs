use chrono::prelude::*;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};


/*
Rules: 1. Previous hash needs to actually match hash of last block in chain
2. Hash start with Difficulty prefix therefore mined correctly, id needs to be the latest ID ++
4. Hash needs to be correct.

*/

// Difficulty on the network
// When mining the person mining has to hash data for block.
const DIFFICULTY_PREFIX: &str = "00";
pub struct App {
    pub blocks: Vec<Block>,
}


// Checks whether a hash fits our difficulty prefix condition
fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();
    for c in hash {
        res.push_str(&format!("{:b}", c));
    }
    res
}

// List of blocks. Add new blocks to the end of this to become blockchain data structure
// The logic will make this list a chain of blocks where each block references the previous block's hash
// Block consists of Id from 0 then a sha256 hash, hash of the previous block, a time stamp, data contained in the block and a nonce.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64, // This is the number that we change to get a hash that satisfies our difficulty criteria.
}

// Implementing App struct

impl App {
    fn new() -> Self 
    {
        Self { blocks: vec![]}
    }
    // Initializing with empty chain
    // Genesis method creates first hard coded block in blockchain 
    fn genesis(&mut self) {
    let genesis_block = Block {
        id: 0,
        timestamp: Utc::now().timestamp(),
        previous_hash: String::from("genesis"),
        data: String::from("genesis!"),
        nonce: 2863,
        hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
    };
        self.blocks.push(genesis_block);
    }
    // Validate wehehter block is valid
    fn try_add_block(&mut self, block: Block){
        let latest_block = self.blocks.last().expect("there is at least one block");
        if self.is_block_valid(&block, latest_block){
            self.blocks.push(block);
        } else {
            error!("Could not add block - invalid");
        }
    }

    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary_representation(
            &hex::decode(&block.hash).expect("can decode from hex"),
        )
        .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("block with id: {} has invalid difficulty", block.id);
            return false;
        } else if block.id != previous_block.id + 1 {
            warn! (
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id

            );
            return false;
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,
        )) != block.hash
        {
            warn!("block with id: {} has invalid hash", block.id);
            return false;
        }
        true
        
    }

    // Validation for whole chain. If one block fails the validation, fail the whole chain
    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len(){
            if i == 0 {
                continue;
            }
            let first = chain.get(i -1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if !self.is_block_valid(second, first){
                return false;
            }
        }
        true
    }
    // Always choose the longest chain
    fn choose_chain(&mut self, local: Vec, remote: Vec) -> Vec {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len(){
                local
            } else {
                remote
            }
        } else if is_remote_valid && !is_local_valid {
            remote
        } else if !is_remote_valid && is_local_valid {
            local
        } else {
            panic!("local and remote chains are both invalid");
        }
    }

    
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let now = Utc::now();
        let (nonce, hash) = mine_block(id, now.timestamp(), &previous_hash, &data);

        Self {
            id,
            hash,
            previous_hash,
            timestamp: now.timestamp(),
            data,
            nonce,
        }
    }
}

fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
    info!("Mining block...");
    let mut nonce = 0;
    loop {
        // Rough progress indicator
        if nonce % 100000 == 0 {
            info!("nonce: {}", nonce);
        }
        // Trying to find a hash that satisfies the difficulty criteria
        let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
        // To check whether the calculated hash adheres difficulty criteria of starting with two zeros
        let binary_hash = hash_to_binary_representation(&hash);
        if binary_hash.starts_with(DIFFICULTY_PREFIX) {
            info!(
                "mined! nonce: {}, hash: {}, binary hash: {}",
                nonce,
                hex::encode(&hash),
                binary_hash
            );
            return (nonce, hex::encode(hash));
        }
        nonce += 1;
    }
}
// Create a json representation of block and put it through sha256 hash function
fn caculate_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
    let data = serde_json::json!({
        "id": id,
        "timestamp": timestamp,
        "previous_hash": previous_hash,
        "data": data,
        "nonce": nonce // this is the number that we change to get a hash that satisfies our difficulty criteria
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}
