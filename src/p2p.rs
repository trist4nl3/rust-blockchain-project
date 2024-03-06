use super::{App, Block};
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, Swarm},
    NetworkBehaviour, PeerId,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc;

// p2p logic will be implemented here
// keys and peer id intrinicis for identifying a client on the network
pub static KEYS: Lazy = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy = Lazy::new(|| PeerId::from(KEYS.public()));
// Using FloodSub protocol a simple pubsub protocol.
// Topics are channels to subscript to, chains to send local block chain to other nodes and receive theirs. And The same for new blocks.

pub static CHAIN_TOPIC: Lazy = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy = Lazy::new(|| Topic::new("blocks"));

// Potentially use point to point request / response model

// More effience to use GossipSub for larger networks


// Struct of what to expect when given local blockchan and use to send them to other nodes
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec,
    pub receiver: String,
}

// Triggers the above interaction if sent with peer_id it will trigger that they send us their chain.
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

// Event Type enum to help send events across the application and keep application state in sync with incoming and outgoing network traffic

pub enum EventType {
    LocalChainRequest(ChainResponse),
    Input(String),
    Init,
}

#[Derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub app: App,
}