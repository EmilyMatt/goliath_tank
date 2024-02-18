use crate::security::{GoliathCert, GoliathPKey};
use rcgen::{date_time_ymd, Certificate, CertificateParams, DistinguishedName, PKCS_RSA_SHA256};
use rsa::{pkcs8::EncodePrivateKey, rand_core::OsRng, RsaPrivateKey};

pub fn generate_certification_and_keys() -> Result<(GoliathCert, GoliathPKey), String> {
    log::info!("Generating a private key");
    // We generate a keypair
    let mut rng = OsRng;
    let pkey = RsaPrivateKey::new(&mut rng, 2048).map_err(|err| err.to_string())?;
    let pk_der = pkey.to_pkcs8_der().map_err(|err| err.to_string())?;
    let key_pair = rcgen::KeyPair::try_from(pk_der.as_bytes()).unwrap();

    log::info!("Generated keypair");

    // Dunno why, it impl's Default but I can't construct it destructure-style so I have to manually do it
    let mut params = CertificateParams::default();
    params.alg = &PKCS_RSA_SHA256;
    params.not_after = date_time_ymd(2012, 1, 1);
    params.not_after = date_time_ymd(2026, 1, 1);
    params.distinguished_name = DistinguishedName::new();
    params.key_pair = Some(key_pair);

    // Generate certificate
    let cert = Certificate::from_params(params).map_err(|err| err.to_string())?;

    log::info!("Generated certificate");

    // Return the certificate, and private key (both serialized)
    Ok((
        GoliathCert(cert.serialize_der().map_err(|err| err.to_string())?),
        GoliathPKey(cert.serialize_private_key_der()),
    ))
}

#[cfg(test)]
mod tests {
    use super::generate_certification_and_keys;
    use tokio_rustls::rustls::{pki_types::PrivateKeyDer, ServerConfig};

    #[test]
    fn test_generate_certificates() {
        let res = generate_certification_and_keys();
        assert!(res.is_ok());

        let (cert, key) = res.unwrap();
        assert!(ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert.0.into()], PrivateKeyDer::Pkcs8(key.0.into()))
            .is_ok());
    }
}
