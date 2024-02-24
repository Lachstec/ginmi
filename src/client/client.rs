use crate::auth::AuthService;
use crate::error::GinmiError;
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use crate::gen::gnmi::get_request::DataType;
use crate::gen::gnmi::{Encoding, CapabilityRequest, GetRequest, GetResponse, Path, PathElem};
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
    /// # use ginmi::Client;
    /// # tokio_test::block_on(async {
    /// # const CERT: &str = "CA Certificate";
    /// let mut client = Client::builder("https://clab-srl01-srl:57400")
    ///     .tls(CERT, "clab-srl01-srl")
    ///     .credentials("admin", "admin")
    ///     .build()
    ///     .await
    ///     .unwrap();
    ///
    /// let capabilities = client.capabilities().await.unwrap();
    /// # });
    /// ```
    pub async fn capabilities(&mut self) -> Result<Capabilities, GinmiError> {
        let req = CapabilityRequest::default();
        let res = self.inner.capabilities(req).await?;
        Ok(Capabilities(res.into_inner()))
    }

    /// Get data from a given gNMI Target device.
    /// according to the [gNMI Specification Section 3.3.1](https://github.com/openconfig/reference/blob/master/rpc/gnmi/gnmi-specification.md#331-the-getrequest-message)
    /// and [gNMI Specification Section 3.3.2](https://github.com/openconfig/reference/blob/master/rpc/gnmi/gnmi-specification.md#332-the-getresponse-message)
    ///
    /// # Examples
    /// t.b.w.
    pub async fn get(&mut self, prefix: &str, path: &str) -> Result<GetResponse, GinmiError> {
        let mut req = GetRequest::default();

        if prefix != "" {
            req.prefix = Some(Path {
                elem: vec![PathElem {
                    name: prefix.to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            });
        }
        req.set_encoding(Encoding::JsonIetf);
        req.set_type(DataType::Config);
        // TODO: need to add a generator for the Path in various formats
        if path.matches('/').count() > 0 {
            let mut path_elems = Vec::new();
            for elem in path.split('/') {
                path_elems.push(PathElem {
                    name: elem.to_string(),
                    ..Default::default()
                });
            }
            req.path.push(Path {
                elem: path_elems,
                ..Default::default()
            });
        } else {
            req.path.push(Path {
                elem: vec![PathElem {
                    name: path.to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            });
        }
        let res = self.inner.get(req).await?;
        //Ok(Notifications(res.into_inner()))
        Ok(res.into_inner())
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

    #[tokio::test]
    async fn test_client_connection() {
        //const CERT: &str = "CA Certificate";

        const CERT: &str = "-----BEGIN CERTIFICATE-----
MIIDfzCCAmegAwIBAgICB+MwDQYJKoZIhvcNAQELBQAwUTELMAkGA1UEBhMCVVMx
CTAHBgNVBAcTADEVMBMGA1UEChMMY29udGFpbmVybGFiMQkwBwYDVQQLEwAxFTAT
BgNVBAMTDHNybDAxIGxhYiBDQTAeFw0yNDAyMTMxOTA4MzJaFw0yNTAyMTMxOTA4
MzJaMFExCzAJBgNVBAYTAlVTMQkwBwYDVQQHEwAxFTATBgNVBAoTDGNvbnRhaW5l
cmxhYjEJMAcGA1UECxMAMRUwEwYDVQQDEwxzcmwwMSBsYWIgQ0EwggEiMA0GCSqG
SIb3DQEBAQUAA4IBDwAwggEKAoIBAQDKQELjOyurWYV6ht/go6zYlegvrFySBIZn
WEKQ6Zv6HEsyLwgY2WgrvfszKhoCWX7Cc8jiyH72+US9tlLKP8yMl/m6qDLXABmM
BkzCpgrPFR1Zm8E2taI/6chlfeO0M2yt7etg+HHSHrKDutXM48doLTHqFsO6yCI6
6w+VG1msjgm2OFvnKKk5MsJ/TbVYc5IPSLEabdjrL7cufDPFnBItI64fL+SrpXLr
g3bVeaKjExXntrGOR+G2LKAPLmRG0HtEVFSrul0fPpScTaD7UkgPps9QX550Hu2L
X01pPcwRoXSHNWy0NNYxaFT0K9pXtn7HQYEYO5X9WCY8QdbO5skFAgMBAAGjYTBf
MA4GA1UdDwEB/wQEAwIChDAdBgNVHSUEFjAUBggrBgEFBQcDAgYIKwYBBQUHAwEw
DwYDVR0TAQH/BAUwAwEB/zAdBgNVHQ4EFgQU9WlaYHnZKkUCmQ6aGmoHNMTPfhcw
DQYJKoZIhvcNAQELBQADggEBAAgMb2DPYdGp2bNe3012mB3NShHII/9OKjHLlqPP
rQ002/qRL5SSaAzQz6m9/LMK5kPYemvmeInP16L2r6nk9BUvk2cuMiS02eUYDP+w
0kJ3oh2xF9XGpVxjh0p8HonR1+EDxlciX/8BXs5cul44VsFlNXdkR9LrGP1WAo68
S5lOwsZm5Zb0Rmj+yQk5XIuU47vP8lqeqHgA+6UM3PWkz6ElaXyUqwa3POGV5mee
qGsputHX3sASxEBKOBfpISuSKAVIJugPpOTWKOig5bmh6tx8T+n52wNA0wq39t0u
vvqc8ppQMQs/qOCtLea7p3GmhwIFCcsH8vIW0Cik9maLPs4=
-----END CERTIFICATE-----";

        let mut client = Client::builder("https://clab-srl01-srl:57400")
            .tls(CERT, "clab-srl01-srl")
            .credentials("admin", "NokiaSrl1!")
            .build()
            .await
            .unwrap();
        
        let capabilities = client.capabilities().await.unwrap();
        //assert_eq!("0.10.0", capabilities.expect("REASON").gnmi_version());
        assert_eq!("0.10.0", capabilities.gnmi_version());

        let notifications = client.get("","system").await.unwrap();
        //assert!(Some(notifications.get_response().get_notification().len()) > Some(0));
        print!("{:?}", notifications);
    }
}
