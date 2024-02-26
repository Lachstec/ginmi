use super::ClientBuilder;
use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use http::Uri;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use tokio_rustls::rustls::{Certificate, ClientConfig, Error, RootCertStore, ServerName};
use tokio_rustls::rustls::client::{ServerCertVerifier, ServerCertVerified};
use crate::{Client, GinmiError};
use crate::gen::gnmi::g_nmi_client::GNmiClient;

pub struct DangerousClientBuilder<'a> {
    builder: ClientBuilder<'a>,
    client_config: Option<ClientConfig>
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

    pub async fn build(mut self) -> Result<Client<hyper::Client<HttpsConnector<HttpConnector>, tonic::body::BoxBody>>, GinmiError> {
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
        
        let client = GNmiClient::with_origin(http_client, uri);
        
        Ok(Client {
            inner: client
        })
    }
}

impl<'a> From<ClientBuilder<'a>> for DangerousClientBuilder<'a> {
    fn from(builder: ClientBuilder<'a>) -> Self {
        DangerousClientBuilder {
            builder,
            client_config: None
        }
    }
}

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(&self, _end_entity: &Certificate, _intermediates: &[Certificate], _server_name: &ServerName, _scts: &mut dyn Iterator<Item=&[u8]>, _ocsp_response: &[u8], _now: SystemTime) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}