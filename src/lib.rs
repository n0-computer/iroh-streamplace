uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct SenderEndpoint {
    endpoint: iroh::Endpoint,
}

#[uniffi::export]
impl SenderEndpoint {
    /// Create a new sender endpoint.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn new() -> SenderEndpoint {
        // TODO: error handling
        let endpoint = iroh::Endpoint::builder().bind().await.unwrap();
        SenderEndpoint { endpoint }
    }
}
