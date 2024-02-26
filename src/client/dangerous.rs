use super::ClientBuilder;
use crate::auth::AuthInterceptor;
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use crate::{Client, GinmiError};
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

type DangerousClient =
    InterceptedService<hyper::Client<HttpsConnector<HttpConnector>, BoxBody>, AuthInterceptor>;

pub struct DangerousClientBuilder<'a> {
    builder: ClientBuilder<'a>,
    client_config: Option<ClientConfig>,
}

impl<'a> DangerousClientBuilder<'a> {
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

    pub async fn build(mut self) -> Result<Client<DangerousClient>, GinmiError> {
        let mut http = HttpConnector::new();
        http.enforce_http(false);

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

        let svc = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(AuthInterceptor::new(
                username, password,
            )))
            .service(http_client);

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
