//! Connect to a gNMI-capable Endpoint without verifying the TLS-Certificate
//!
//! Provides the means to connect to gNMI-capable Devices without verifying the
//! used TLS-Certificates. This is accomplished by manually providing a [`ClientConfig`]
//! to a hyper client and using it as a replacement for the default [Channel]. The
//! [`ClientConfig`] is configured with a custom [`ServerCertVerifier`]
//! that will always return a successful validation.
//!
//! [Channel]: tonic::transport::Channel
//!
//! # Safety
//! You should never use the functionality provided by this module, except when you need to test
//! something locally and do not care for the certificates. Using this in the wild is very dangerous because
//! you are susceptible to Man-in-the-Middle attacks.
//!
//! # Examples
//! Connecting to a SR-Linux device, ignoring any validation issues that happen with its certificate:
//! ```rust
//! # use ginmi::client::Client;
//! # fn main() -> std::io::Result<()> {
//! # tokio_test::block_on(async {
//! # const CA_CERT: &str = "CA Certificate";
//! let mut client = Client::builder("https://clab-srl01-srl:57400")
//!     .credentials("admin", "password1")
//!     .dangerous()
//!     .disable_certificate_validation()
//!     .build()
//!     .await?;
//! # })}
use super::ClientBuilder;
use crate::auth::AuthInterceptor;
use crate::client::Client;
use crate::error::GinmiError;
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use http::Uri;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio_rustls::rustls::client::{ServerCertVerified, ServerCertVerifier};
use tokio_rustls::rustls::{Certificate, ClientConfig, Error, RootCertStore, ServerName};
use tonic::body::BoxBody;
use tonic::codegen::InterceptedService;
use tonic::metadata::AsciiMetadataValue;

pub type DangerousConnection =
    InterceptedService<hyper::Client<HttpsConnector<HttpConnector>, BoxBody>, AuthInterceptor>;

/// Builder for [`Client`]s with extra options that are dangerous and require extra care.
pub struct DangerousClientBuilder<'a> {
    builder: ClientBuilder<'a>,
    client_config: Option<ClientConfig>,
}

impl<'a> DangerousClientBuilder<'a> {
    /// Disable the verification of TLS-certificates
    ///
    /// # Safety
    /// Using this option completely disables certificate validation which on turn
    /// makes you susceptible to Man-in-the-Middle attacks. This option can be useful for local
    /// testing purposes, but should be avoided at all cost for any other use case.
    pub fn disable_certificate_verification(mut self) -> Self {
        let roots = RootCertStore::empty();

        let mut tls = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        tls.dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));

        self.client_config = Some(tls);
        self
    }

    /// Consume the [`DangerousClientBuilder`] and return a [`Client`].
    ///
    /// # Errors
    /// - Returns [`GinmiError::InvalidUriError`] if specified target is not a valid URI.
    /// - Returns [`GinmiError::TransportError`] if the TLS-Settings are invalid.
    /// - Returns [`GinmiError::TransportError`] if a connection to the target could not be
    /// established.
    pub async fn build(self) -> Result<Client<DangerousConnection>, GinmiError> {
        // create a hyper HttpConnector
        let mut http = HttpConnector::new();
        http.enforce_http(false);

        // specify tls configuration for the http connector to enable https
        let connector = tower::ServiceBuilder::new()
            .layer_fn(move |s| {
                let tls = self.client_config.clone().unwrap();

                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_tls_config(tls)
                    .https_or_http()
                    .enable_http2()
                    .wrap_connector(s)
            })
            .service(http);

        // create a hyper client from the connector
        let http_client = hyper::Client::builder().build(connector);

        let uri = match Uri::from_str(self.builder.target) {
            Ok(u) => u,
            Err(e) => return Err(GinmiError::InvalidUriError(e.to_string())),
        };

        let (username, password) = match self.builder.creds {
            Some(c) => (
                Some(AsciiMetadataValue::from_str(c.username)?),
                Some(AsciiMetadataValue::from_str(c.password)?),
            ),
            None => (None, None),
        };

        // add the authentication interceptor to the service.
        let svc = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(AuthInterceptor::new(
                username, password,
            )))
            .service(http_client);

        // create a client, overriding the default uri with the uri in the builder
        let client = GNmiClient::with_origin(svc, uri);

        Ok(Client { inner: client })
    }
}

impl<'a> From<ClientBuilder<'a>> for DangerousClientBuilder<'a> {
    fn from(builder: ClientBuilder<'a>) -> Self {
        DangerousClientBuilder {
            builder,
            client_config: None,
        }
    }
}

#[derive(Debug)]
/// ServerCertVerifier that always returns a successful certificate validation regardless of the reality.
///
/// This Verifier performs no actual Verification at all.
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
