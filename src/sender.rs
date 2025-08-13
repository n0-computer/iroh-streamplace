use std::collections::HashMap;

use bytes::Bytes;
use iroh::NodeId;
use snafu::{OptionExt, ResultExt};
use tokio::sync::Mutex;

use crate::ALPN;
use crate::endpoint::Endpoint;
use crate::error::{
    Error, MissingConnectionSnafu, NewConnectionSnafu, OpenStreamSnafu, SendMessageSnafu,
};
use crate::key::PublicKey;
use crate::utils::NodeAddr;

#[derive(uniffi::Object)]
pub struct Sender {
    endpoint: Endpoint,
    connections: Mutex<HashMap<NodeId, imsg::Connection>>,
}

#[uniffi::export]
impl Sender {
    /// Create a new sender.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn new(endpoint: &Endpoint) -> Result<Sender, Error> {
        Ok(Sender {
            endpoint: endpoint.clone(),
            connections: Default::default(),
        })
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn add_peer(&self, addr: &NodeAddr) -> Result<(), Error> {
        let addr: iroh::NodeAddr = addr.clone().try_into()?;

        let mut conns = self.connections.lock().await;
        let node_id = addr.node_id;
        if conns.contains_key(&node_id) {
            return Ok(());
        }
        let conn = self.endpoint.endpoint.connect(addr, ALPN).await?;
        let conn = imsg::Connection::new(conn)
            .await
            .context(NewConnectionSnafu)?;
        conns.insert(node_id, conn);
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn send(&self, node_id: &PublicKey, data: &[u8]) -> Result<(), Error> {
        let node_id: NodeId = node_id.into();
        let conn = self
            .connections
            .lock()
            .await
            .get(&node_id)
            .cloned()
            .context(MissingConnectionSnafu)?;

        // TODO: store streams
        let stream = conn.open_stream().await.context(OpenStreamSnafu)?;
        let data = Bytes::copy_from_slice(data);
        stream.send_msg(data).await.context(SendMessageSnafu)?;
        Ok(())
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn node_addr(&self) -> NodeAddr {
        self.endpoint.node_addr().await
    }
}
