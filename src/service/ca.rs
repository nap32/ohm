use http::uri::Authority;
use hyper::{Body, Request};
use tokio_rustls::rustls;
use tokio_rustls::rustls::ServerConfig;

use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rand;
use openssl::x509::extension::SubjectAlternativeName;
use openssl::x509::{X509Builder, X509NameBuilder, X509};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct CA {
    ca_cert: X509,
    signing_key: PKey<Private>,
}

impl CA {
    pub async fn new() -> Self {
        let private_key_bytes: &[u8] =
            &std::fs::read(&crate::CONFIG.get().unwrap().ca.key_relative_path).unwrap();
        let pkey =
            PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");
        let ca_cert_bytes: &[u8] =
            &std::fs::read(&crate::CONFIG.get().unwrap().ca.pem_relative_path).unwrap();
        let cert = X509::from_pem(ca_cert_bytes).expect("Failed to parse CA certificate pem.");

        Self {
            ca_cert: cert,
            signing_key: pkey,
        }
    }

    pub async fn get_proxy_config(
        &mut self,
        request: Request<Body>,
    ) -> Result<ServerConfig, Error> {
        let authority = request
            .uri()
            .authority()
            .expect("URI does not contain authority");

        self.create_server_config(authority).await
    }

    async fn create_server_config(&mut self, authority: &Authority) -> Result<ServerConfig, Error> {
        let result = self.create_proxy_certificate(authority).await;
        let cert: rustls::Certificate;
        match result {
            Ok(t) => {
                cert = t;
            }
            Err(e) => return Err(e),
        }

        let private_key = rustls::PrivateKey(
            self.signing_key
                .private_key_to_der()
                .expect("Failed to encode private key."),
        );

        let mut server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], private_key.clone())
            .expect("Failed to build ServerConfig.");

        server_config.alpn_protocols = vec![
            #[cfg(feature = "http2")]
            b"h2".to_vec(),
            b"http/1.1".to_vec(),
        ];

        Ok(server_config)
    }

    async fn create_proxy_certificate(
        &mut self,
        authority: &Authority,
    ) -> Result<rustls::Certificate, Error> {
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_text("C", "US").unwrap();
        name_builder.append_entry_by_text("ST", "CA").unwrap();
        name_builder.append_entry_by_text("O", "OHM").unwrap();
        name_builder
            .append_entry_by_text("CN", authority.host())
            .unwrap();
        let name = name_builder.build();

        let mut x509_builder = X509Builder::new().unwrap();
        x509_builder.set_subject_name(&name)?;
        x509_builder.set_version(2)?;

        let not_before = Asn1Time::days_from_now(0)?;
        x509_builder.set_not_before(&not_before)?;
        let not_after = Asn1Time::days_from_now(365)?;
        x509_builder.set_not_after(&not_after)?;

        x509_builder.set_pubkey(&self.signing_key)?;
        x509_builder.set_issuer_name(self.ca_cert.subject_name())?;

        let alternative_name = SubjectAlternativeName::new()
            .dns(authority.host())
            .build(&x509_builder.x509v3_context(Some(&self.ca_cert), None))?;
        x509_builder.append_extension(alternative_name)?;

        let mut serial_number = [0; 16];
        rand::rand_bytes(&mut serial_number)?;
        let serial_number = BigNum::from_slice(&serial_number)?;
        let serial_number = Asn1Integer::from_bn(&serial_number)?;
        x509_builder.set_serial_number(&serial_number)?;

        x509_builder.sign(&self.signing_key, MessageDigest::sha256())?;
        let x509 = x509_builder.build();

        Ok(rustls::Certificate(x509.to_der()?))
    }
}
