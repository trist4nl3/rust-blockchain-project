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

// App behaviour implements network behaviour, which is libp2p concept for implementing a decentralized network stack.
// App behaviour holds our FloodSub instacne for pub/sub cmmunication and Mdns instance, which will enalbe us to automatically find other nodes on local network

// Add blockchain ap as well as channels.
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

impl AppBehaviour {
    pub async fn new(
        app: App,
        response_sender: mpsc::UnboundedSender,
        init_sender: mpsc::UnboundedSender,
    ) -> Self {
        let mut behaviour = Self {
            app,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),
            response_sender,
            init_sender,
        };
        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

// Handling incoming messages

impl NetworkBehaviourEventProcess<MdnsEvent> for AppBehaviour {
    fn inject_event(&mut self, event: MdnsEvent){
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    
                }
            }
        }
    }
}