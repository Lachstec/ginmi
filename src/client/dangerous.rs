use super::ClientBuilder;
use std::convert::From;
use std::sync::Arc;
use std::time::SystemTime;
use tokio_rustls::rustls::{Certificate, ClientConfig, Error, RootCertStore, ServerName};
use tokio_rustls::rustls::client::{ServerCertVerifier, ServerCertVerified};

pub struct DangerousClientBuilder<'a> {
    builder: ClientBuilder<'a>,
}

impl<'a> DangerousClientBuilder<'a> {
    pub fn disable_certificate_verification(mut self) -> ClientBuilder<'a> {
        let roots = RootCertStore::empty();

        let mut tls = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();
        
        tls.dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
        
        
        self.builder.client_config = Some(tls);
        self.builder
    }
}

impl<'a> From<ClientBuilder<'a>> for DangerousClientBuilder<'a> {
    fn from(builder: ClientBuilder<'a>) -> Self {
        DangerousClientBuilder {
            builder
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