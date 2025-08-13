/// An Error.
#[derive(Debug, snafu::Snafu, uniffi::Error)]
#[uniffi(flat_error)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Bind failure"), context(false))]
    IrohBind { source: iroh::endpoint::BindError },
    #[snafu(display("Invalid URL"), context(false))]
    InvalidUrl { source: url::ParseError },
    #[snafu(display("Failed to connect"), context(false))]
    IrohConnect {
        source: iroh::endpoint::ConnectError,
    },
    #[snafu(display("Invalid network address"), context(false))]
    InvalidNetworkAddress { source: std::net::AddrParseError },
    #[snafu(display("Failed to open stream"))]
    OpenStream { source: anyhow::Error },
    #[snafu(display("Failed to send message"))]
    SendMessage { source: imsg::StreamError },
    #[snafu(display("Failed to create connection"))]
    NewConnection { source: anyhow::Error },
    #[snafu(display("No connection available"))]
    MissingConnection,
    #[snafu(display("Invalid public key"))]
    InvalidPublicKey,
}
