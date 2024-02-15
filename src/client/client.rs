use crate::auth::AuthService;
use crate::error::GinmiError;
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use crate::gen::gnmi::CapabilityRequest;
use super::capabilities::Capabilities;
use http::HeaderValue;
use std::str::FromStr;
use std::sync::Arc;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Uri};

/// Provides the main functionality of connection to a target device
/// and manipulating configuration or querying telemetry.
#[derive(Debug, Clone)]
pub struct Client {
    inner: GNmiClient<AuthService<Channel>>,
}

impl<'a> Client {
    /// Create a [`ClientBuilder`] that can create [`Client`]s.
    pub fn builder(target: &'a str) -> ClientBuilder<'a> {
        ClientBuilder::new(target)
    }

    /// Returns information from the target device about its capabilities
    /// according to the [gNMI Specification Section 3.2.2](https://github.com/openconfig/reference/blob/master/rpc/gnmi/gnmi-specification.md#322-the-capabilityresponse-message)
    ///
    /// # Examples
    /// ```rust
    /// # use ginmi::Client
    /// # tokio_test::block_on({ async
    /// let mut client = Client::builder("https://clab-srl01-srl:57400")
    ///     .tls(CERT, "clab-srl01-srl")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let capabilities = client.capabilities().await;
    /// # }
    /// ```
    pub async fn capabilities(&mut self) -> Capabilities {
        let req = CapabilityRequest::default();
        match self.inner.capabilities(req).await {
            Ok(val) => Capabilities(val.into_inner()),
            Err(e) => panic!("Error getting capabilities: {:?}", e),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
}

/// Builder for [`Client`]s
///
/// Used to configure and create instances of [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder<'a> {
    target: &'a str,
    creds: Option<Credentials<'a>>,
    tls_settings: Option<ClientTlsConfig>,
}

impl<'a> ClientBuilder<'a> {
    pub fn new(target: &'a str) -> Self {
        Self {
            target,
            creds: None,
            tls_settings: None,
        }
    }

    /// Configure credentials to use for connecting to the target device.
    pub fn credentials(mut self, username: &'a str, password: &'a str) -> Self {
        self.creds = Some(Credentials { username, password });
        self
    }

    /// Configure TLS to use for connecting to the target device.
    pub fn tls(mut self, ca_certificate: impl AsRef<[u8]>, domain_name: impl Into<String>) -> Self {
        let cert = Certificate::from_pem(ca_certificate);
        let settings = ClientTlsConfig::new()
            .ca_certificate(cert)
            .domain_name(domain_name);
        self.tls_settings = Some(settings);
        self
    }

    /// Consume the [`ClientBuilder`] and return a [`Client`].
    ///
    /// # Errors
    /// - Returns [`GinmiError::InvalidUriError`] if specified target is not a valid URI.
    /// - Returns [`GinmiError::TransportError`] if the TLS-Settings are invalid.
    /// - Returns [`GinmiError::TransportError`] if a connection to the target could not be
    /// established.
    pub async fn build(self) -> Result<Client, GinmiError> {
        let uri = match Uri::from_str(self.target) {
            Ok(u) => u,
            Err(e) => return Err(GinmiError::InvalidUriError(e.to_string())),
        };

        let mut endpoint = Channel::builder(uri);

        if self.tls_settings.is_some() {
            endpoint = endpoint.tls_config(self.tls_settings.unwrap())?;
        }

        let channel = endpoint.connect().await?;

        return if let Some(creds) = self.creds {
            let user_header = HeaderValue::from_str(creds.username)?;
            let pass_header = HeaderValue::from_str(creds.password)?;
            Ok(Client {
                inner: GNmiClient::new(AuthService::new(
                    channel,
                    Some(Arc::new(user_header)),
                    Some(Arc::new(pass_header)),
                )),
            })
        } else {
            Ok(Client {
                inner: GNmiClient::new(AuthService::new(channel, None, None)),
            })
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_uri() {
        let client = Client::builder("$$$$").build().await;
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn invalid_tls_settings() {
        let client = Client::builder("https://test:57400")
            .tls("invalid cert", "invalid domain")
            .build()
            .await;
        assert!(client.is_err());
    }
}
