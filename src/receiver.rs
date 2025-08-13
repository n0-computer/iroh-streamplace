use std::sync::Arc;

use iroh::Watcher;
use n0_future::task::{self, AbortOnDropHandle};
use tokio::task::JoinSet;
use tracing::{debug, warn};

use crate::ALPN;
use crate::error::Error;
use crate::key::PublicKey;
use crate::utils::NodeAddr;

#[derive(uniffi::Object)]
pub struct ReceiverEndpoint {
    endpoint: iroh::Endpoint,
    _handle: AbortOnDropHandle<()>,
}

#[uniffi::export]
impl ReceiverEndpoint {
    /// Create a new receiver endpoint.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn new(handler: Arc<dyn DataHandler>) -> Result<ReceiverEndpoint, Error> {
        let endpoint = iroh::Endpoint::builder()
            .discovery_n0()
            .discovery_local_network()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;

        let ep = endpoint.clone();
        let handle = task::spawn(async move {
            let mut tasks = JoinSet::default();

            while let Some(incoming) = ep.accept().await {
                let handler = handler.clone();
                tasks.spawn(async move {
                    let Ok(conn) = incoming.await else {
                        return;
                    };
                    let peer = Arc::new(PublicKey::from(
                        conn.remote_node_id().expect("invalid remote"),
                    ));
                    let conn = match imsg::Connection::new(conn).await {
                        Ok(conn) => conn,
                        Err(err) => {
                            warn!("imsg connection failed: {:?}", err);
                            return;
                        }
                    };

                    while let Ok(stream) = conn.accept_stream().await {
                        debug!("accepted stream");
                        if let Ok(msg) = stream.recv_msg().await {
                            debug!("received msg {} bytes", msg.len());
                            handler
                                .clone()
                                .handle_data(peer.clone(), msg.to_vec())
                                .await;
                        }
                    }
                });
            }

            // cleanup
            tasks.abort_all();
        });

        Ok(ReceiverEndpoint {
            endpoint,
            _handle: AbortOnDropHandle::new(handle),
        })
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn node_addr(&self) -> NodeAddr {
        let _ = self.endpoint.home_relay().initialized().await;
        let addr = self.endpoint.node_addr().initialized().await;
        addr.into()
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait DataHandler: Send + Sync {
    async fn handle_data(&self, peer: Arc<PublicKey>, data: Vec<u8>);
}

#[cfg(test)]
mod tests {

    use crate::sender::SenderEndpoint;

    use super::*;

    #[tokio::test]
    async fn test_roundtrip() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        let sender = SenderEndpoint::new().await.unwrap();

        let (s, mut r) = tokio::sync::mpsc::channel(5);

        #[derive(Debug, Clone)]
        struct TestHandler {
            messages: tokio::sync::mpsc::Sender<(PublicKey, Vec<u8>)>,
        }

        #[async_trait::async_trait]
        impl DataHandler for TestHandler {
            async fn handle_data(&self, peer: Arc<PublicKey>, data: Vec<u8>) {
                self.messages
                    .send((peer.as_ref().clone(), data))
                    .await
                    .unwrap();
            }
        }

        let handler = TestHandler { messages: s };
        let receiver = ReceiverEndpoint::new(Arc::new(handler.clone()))
            .await
            .unwrap();

        let receiver_addr = receiver.node_addr().await;
        println!("recv addr: {:?}", receiver_addr);
        let receiver_id = receiver_addr.node_id();

        // add peer
        sender
            .add_peer(&NodeAddr::new(&receiver_id, None, Vec::new()))
            .await
            .unwrap();

        // send a few messages
        for i in 0u8..5 {
            sender.send(&receiver_id, &[i, 0, 0, 0]).await.unwrap();
        }

        // make sure the receiver got them
        let sender_id = sender.node_addr().await.node_id();
        for i in 0u8..5 {
            let (id, msg) = r.recv().await.unwrap();
            assert_eq!(id, sender_id);
            assert_eq!(msg, vec![i, 0, 0, 0]);
        }
    }
}
