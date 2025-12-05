use crate::types::{ConsensusInput, HybridTx, NarwhalBatch};
use anyhow::Result;
use libp2p::{
    gossipsub::{self, Gossipsub, GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use libp2p::Transport;
use libp2p::swarm::NetworkBehaviour;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tracing::{info, warn};

const TOPIC_TX: &str = "eth-narwhal-tx";
const TOPIC_BATCH: &str = "eth-narwhal-batch";

#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    gossipsub: Gossipsub,
}

#[derive(Debug, Serialize, Deserialize)]
enum GossipMessage {
    Tx(HybridTx),
    Batch(NarwhalBatch),
}

pub async fn spawn_p2p(
    listen_addr: &str,
    consensus_tx: Sender<ConsensusInput>,
) -> Result<()> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    info!("Local peer id: {local_peer_id}");

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1Lazy)
        .authenticate(libp2p::noise::Config::new(
            &local_key
        )?)
        .multiplex(yamux::Config::default())
        .boxed();

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .build()
        .expect("valid gossipsub config");

    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;

    let tx_topic = IdentTopic::new(TOPIC_TX);
    let batch_topic = IdentTopic::new(TOPIC_BATCH);

    gossipsub.subscribe(&tx_topic)?;
    gossipsub.subscribe(&batch_topic)?;

    let behaviour = NodeBehaviour { gossipsub };

    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id)
        .build();

    swarm.listen_on(listen_addr.parse()?)?;

    tokio::spawn(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                    GossipsubEvent::Message { message, .. },
                )) => {
                    if let Ok(msg) = serde_json::from_slice::<GossipMessage>(&message.data) {
                        match msg {
                            GossipMessage::Tx(tx) => {
                                let _ = consensus_tx
                                    .send(ConsensusInput::NewTx(tx))
                                    .await;
                            }
                            GossipMessage::Batch(batch) => {
                                let _ = consensus_tx
                                    .send(ConsensusInput::NarwhalBatch(batch))
                                    .await;
                            }
                        }
                    }
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {address}");
                }
                e => {
                    if cfg!(debug_assertions) {
                        warn!("Swarm event: {e:?}");
                    }
                }
            }
        }
    });

    Ok(())
}

