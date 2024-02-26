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
use tonic::body::BoxBody;
use tonic::codegen::InterceptedService;
use crate::{Client, GinmiError};
use crate::gen::gnmi::g_nmi_client::GNmiClient;
use crate::auth::AuthInterceptor;
use crate::client::dangerous::service::AuthSvc;

type DangerousClient = InterceptedService<hyper::Client<HttpsConnector<HttpConnector>, BoxBody>, AuthInterceptor>;

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

        let svc = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(AuthInterceptor::new(None, None)))
            .service(http_client);

        let client = GNmiClient::with_origin(svc, uri);
        
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

mod service {
    use http::{Request, Response};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tonic::body::BoxBody;
    use tonic::transport::Body;
    use tonic::transport::Channel;
    use tower::Service;

    pub struct AuthSvc {
        inner: Channel,
    }

    impl AuthSvc {
        pub fn new(inner: Channel) -> Self {
            AuthSvc { inner }
        }
    }

    impl Service<Request<BoxBody>> for AuthSvc {
        type Response = Response<Body>;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        #[allow(clippy::type_complexity)]
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx).map_err(Into::into)
        }

        fn call(&mut self, req: Request<BoxBody>) -> Self::Future {
            // This is necessary because tonic internally uses `tower::buffer::Buffer`.
            // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
            // for details on why this is necessary
            let clone = self.inner.clone();
            let mut inner = std::mem::replace(&mut self.inner, clone);

            Box::pin(async move {
                // Do extra async work here...
                let response = inner.call(req).await?;

                Ok(response)
            })
        }
    }
}