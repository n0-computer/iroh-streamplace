use std::collections::HashMap;

use bytes::Bytes;
use iroh::{NodeId, Watcher};
use tokio::sync::Mutex;

use crate::ALPN;
use crate::key::PublicKey;
use crate::utils::NodeAddr;

#[derive(uniffi::Object)]
pub struct SenderEndpoint {
    endpoint: iroh::Endpoint,
    connections: Mutex<HashMap<NodeId, imsg::Connection>>,
}

#[uniffi::export]
impl SenderEndpoint {
    /// Create a new sender endpoint.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn new() -> SenderEndpoint {
        // TODO: error handling
        let endpoint = iroh::Endpoint::builder()
            .discovery_n0()
            .discovery_local_network()
            .bind()
            .await
            .unwrap();
        SenderEndpoint {
            endpoint,
            connections: Default::default(),
        }
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_peer(&self, addr: &NodeAddr) {
        let addr: iroh::NodeAddr = addr.clone().try_into().unwrap();

        let mut conns = self.connections.lock().await;
        let node_id = addr.node_id;
        if conns.contains_key(&node_id) {
            return;
        }
        let conn = self.endpoint.connect(addr, ALPN).await.unwrap();
        let conn = imsg::Connection::new(conn).await.unwrap();
        conns.insert(node_id, conn);
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn send(&self, node_id: &PublicKey, data: &[u8]) {
        let node_id: NodeId = node_id.into();
        let Some(conn) = self.connections.lock().await.get(&node_id).cloned() else {
            panic!("no connection");
        };

        // TODO: store streams
        let stream = conn.open_stream().await.unwrap();
        let data = Bytes::copy_from_slice(data);
        stream.send_msg(data).await.unwrap();
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn node_addr(&self) -> NodeAddr {
        let _ = self.endpoint.home_relay().initialized().await;
        let addr = self.endpoint.node_addr().initialized().await;
        addr.into()
    }
}
