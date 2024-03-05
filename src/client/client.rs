use super::capabilities::Capabilities;
#[cfg(feature = "dangerous_configuration")]
use super::dangerous::DangerousClientBuilder;
use crate::auth::AuthInterceptor;
use crate::error::GinmiError;
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use crate::gen::gnmi::{
    CapabilityRequest, Encoding, GetRequest, GetResponse, ModelData, Path, PathElem
};
// remove internal data types of gnmi
use crate::gen::gnmi::get_request::DataType;
use crate::gen::gnmi_ext::Extension;
use hyper::body::Bytes;
use std::str::FromStr;
use tonic::codegen::{Body, InterceptedService, StdError};
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Uri};

/// Provides the main functionality of connection to a target device
/// and manipulating configuration or querying telemetry.
#[derive(Debug, Clone)]
pub struct Client<T> {
    pub(crate) inner: GNmiClient<T>,
}

impl<'a> Client<InterceptedService<Channel, AuthInterceptor>> {
    /// Create a [`ClientBuilder`] that can create [`Client`]s.
    pub fn builder(target: &'a str) -> ClientBuilder<'a> {
        ClientBuilder::new(target)
    }
}

impl<T> Client<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    fn get_path_from_str(path: &str) -> Path {
        // Create a gMNI path from a string
    
        // TODO: need to add a generator for the Path in various formats
        // e.g., /interfaces/interface[name=eth0]/state/counters
        //       openconfig:interfaces:interface[name=eth0]:state:counters
        //       interfaces/interface[name=eth0]/state/counters
        //
        // also need to handle attributes in XPath format (e.g., [name=eth0])
        let mut path_elems = Vec::new();
        if path.matches('/').count() > 0 {
            for elem in path.split('/') {
                path_elems.push(PathElem {
                    name: elem.to_string(),
                    ..Default::default()
                });
            }
        } else {
            path_elems.push(PathElem {
                name: path.to_string(),
                ..Default::default()
            });
        }
        Path {
            elem: path_elems,
            ..Default::default()
        }
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
    pub async fn get(
        &mut self,
        prefix: &str,
        path: &str,
        data_type: DataType,
        encoding: Encoding,
        use_models: Vec<ModelData>,
        extensions: Vec<Extension>,
    ) -> Result<GetResponse, GinmiError> {
        let mut req = GetRequest::default();

        if prefix != "" {
            req.prefix = Some(Self::get_path_from_str(prefix));
        }
        req.path.push(Self::get_path_from_str(path));
        req.set_type(data_type);
        req.set_encoding(encoding);
        for use_model in use_models {
            req.use_models.push(use_model);
        }
        for extension in extensions {
            req.extension.push(extension);
        }
        let res = self.inner.get(req).await?;
        //Ok(Notifications(res.into_inner()))
        Ok(res.into_inner())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Credentials<'a> {
    pub(crate) username: &'a str,
    pub(crate) password: &'a str,
}

/// Builder for [`Client`]s
///
/// Used to configure and create instances of [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder<'a> {
    pub(crate) target: &'a str,
    pub(crate) creds: Option<Credentials<'a>>,
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

    #[cfg(feature = "dangerous_configuration")]
    #[cfg_attr(docsrs, doc(cfg(feature = "dangerous_configuration")))]
    /// Access configuration options that are dangerous and require extra care.
    pub fn dangerous(self) -> DangerousClientBuilder<'a> {
        DangerousClientBuilder::from(self)
    }

    /// Consume the [`ClientBuilder`] and return a [`Client`].
    ///
    /// # Errors
    /// - Returns [`GinmiError::InvalidUriError`] if specified target is not a valid URI.
    /// - Returns [`GinmiError::TransportError`] if the TLS-Settings are invalid.
    /// - Returns [`GinmiError::TransportError`] if a connection to the target could not be
    /// established.
    pub async fn build(
        self,
    ) -> Result<Client<InterceptedService<Channel, AuthInterceptor>>, GinmiError> {
        let uri = match Uri::from_str(self.target) {
            Ok(u) => u,
            Err(e) => return Err(GinmiError::InvalidUriError(e.to_string())),
        };

        let mut endpoint = Channel::builder(uri);

        if self.tls_settings.is_some() {
            endpoint = endpoint.tls_config(self.tls_settings.unwrap())?;
        }

        let channel = endpoint.connect().await?;
        let (username, password) = match self.creds {
            Some(c) => (
                Some(AsciiMetadataValue::from_str(c.username)?),
                Some(AsciiMetadataValue::from_str(c.password)?),
            ),
            None => (None, None),
        };

        Ok(Client {
            inner: GNmiClient::with_interceptor(channel, AuthInterceptor::new(username, password)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_uri() {
        let client = Client::<InterceptedService<Channel, AuthInterceptor>>::builder("$$$$")
            .build()
            .await;
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
        let mut client = Client::builder("https://clab-srl01-srl:57400")
            .credentials("admin", "NokiaSrl1!")
            .dangerous()
            .disable_certificate_verification()
            .build()
            .await
            .unwrap();

        let capabilities = client.capabilities().await.unwrap();
        //assert_eq!("0.10.0", capabilities.expect("REASON").gnmi_version());
        assert_eq!("0.10.0", capabilities.gnmi_version());

        let notifications = client
            .get(
                "",
                "system",
                DataType::Config,
                Encoding::JsonIetf,
                Vec::<ModelData>::new(),
                Vec::<Extension>::new(),
            )
            .await
            .unwrap();
        //assert!(Some(notifications.get_response().get_notification().len()) > Some(0));
        print!("{:?}", notifications);
    }
}