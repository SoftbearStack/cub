// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::common::{AuthenticatedId, CubConfig, Error, Identity, UserName};
use crate::time_id::{NonZeroUnixSeconds, UnixTime};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

// RS256 is for asymmetric, HS256 is symmetric.
const DEFAULT_ALGORITHM: &str = "RS256";

/// JWT validation client.
#[derive(Debug, Default)]
pub struct JwtClient {
    algorithms: HashMap<String, String>,
    private_key_pem: Option<String>,
    public_key_pems: HashMap<String, String>,
}

/// Creates a JWT.
pub fn create_jwt<T: Serialize>(
    client: &JwtClient,
    claims: T,
    ttl_seconds: u64,
) -> Result<String, Error> {
    // The next two errors mapped below never happen.
    let s = serde_json::to_string(&claims)
        .map_err(|e| Error::String(format!("cannot ser claims to JSON str: {e:?}")))?;
    let mut value: Value = serde_json::from_str(&s)
        .map_err(|e| Error::String(format!("cannot de claims into JSON: {e:?}")))?;
    let Value::Object(ref mut claims_obj) = value else {
        return Err(Error::String("claims not an object".to_string()))?;
    };
    let now = NonZeroUnixSeconds::now();
    let iat: u64 = now.0.into();
    let exp: u64 = now.add_seconds(ttl_seconds).0.into();
    claims_obj.insert("iat".to_string(), Value::Number(iat.into()));
    claims_obj.insert("exp".to_string(), Value::Number(exp.into()));
    let Some(ref private_key_pem) = client.private_key_pem else {
        return Err(Error::String(
            "cannot create JWT without a private key".to_string(),
        ));
    };
    let algorithm = client
        .algorithms
        .get(&"default".to_string())
        .map(|s| s.to_owned())
        .unwrap_or(DEFAULT_ALGORITHM.to_string());
    let algorithm = Algorithm::from_str(&algorithm).map_err(|_| {
        Error::String(format!(
            "{algorithm}: cannot generate JWT with this algorithm"
        ))
    })?;
    let mut header = Header::default();
    header.alg = algorithm;
    encode(
        &header,
        &value,
        &EncodingKey::from_rsa_pem(&private_key_pem.clone().into_bytes())
            .map_err(|e| Error::String(format!("Cannot parse private key: {e:?}")))?,
    )
    .map_err(|e| Error::String(format!("cannot create JWT: {e:?}")))
}

/// Decodes and validates a JWT.
fn decode_token<T: DeserializeOwned>(
    jw_token: &str,
    public_key_pem: &str,
    algorithm: &str,
) -> Result<T, Error> {
    let algorithm = Algorithm::from_str(algorithm).map_err(|_| {
        Error::String(format!(
            "{algorithm}: cannot validate JWT with this algorithm"
        ))
    })?;
    let mut validation = Validation::new(algorithm);
    validation.leeway = 30 * 24 * 60 * 60; // For now, not strict about expiration.
    Ok(decode::<T>(
        &jw_token,
        &DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| Error::String(format!("Cannot parse public key: {e:?}")))?,
        &validation,
    )
    .map_err(|e| Error::String(format!("cannot validate JWT token: {e:?}")))?
    .claims)
}

/// Creates a JWT client.
pub fn new_jwt_client(cub_config: &CubConfig) -> JwtClient {
    #[derive(Default, Deserialize)]
    struct JwtConfig {
        #[serde(default)]
        algorithms: HashMap<String, String>,
        #[serde(default)]
        private_key_pem: Option<String>,
        #[serde(default)]
        public_key_pems: HashMap<String, String>,
    }
    #[derive(Deserialize)]
    struct ConfigToml {
        #[serde(default)]
        jwt: JwtConfig,
    }
    match cub_config.get().map(
        |ConfigToml {
             jwt:
                 JwtConfig {
                     algorithms,
                     private_key_pem,
                     public_key_pems,
                 },
         }| JwtClient {
            algorithms,
            private_key_pem,
            public_key_pems,
        },
    ) {
        Ok(config) => config,
        Err(e) => panic!("cannot parse JWT toml: {e:?}"),
    }
}

/// Validates a JSON web token and returns claims of any type.
pub fn validate_jwt<T: DeserializeOwned>(
    client: &JwtClient,
    jw_token: &str,
    provider: Option<&str>,
) -> Result<T, Error> {
    let provider = provider.unwrap_or("default");
    let Some(public_key_pem) = client.public_key_pems.get(&provider.to_string()) else {
        return Err(Error::String(format!(
            "cannot validate JWT without a public key for {provider} provider {:?}",
            client.public_key_pems,
        )));
    };
    let algorithm = client
        .algorithms
        .get(&provider.to_string())
        .map(|s| s.to_owned())
        .unwrap_or(DEFAULT_ALGORITHM.to_string());
    let mut claims: Value = decode_token(jw_token, &public_key_pem, &algorithm)?;
    let Value::Object(ref mut claims_obj) = claims else {
        return Err(Error::String("claims not an object".to_string()))?;
    };
    claims_obj.remove(&"exp".to_string());
    claims_obj.remove(&"iat".to_string());
    // The 2 errors mapped below never happen.
    let s = serde_json::to_string(&claims)
        .map_err(|e| Error::String(format!("cannot ser after rm exp and iat: {e:?}")))?;
    Ok(serde_json::from_str(&s)
        .map_err(|e| Error::String(format!("cannot de after rm exp and iat: {e:?}")))?)
}

/// Validates a JSON web token and return its claims as an `Identity`.
pub fn validate_jwt_identity(
    _client: &JwtClient,
    jw_token: &str,
    provider: &str,
) -> Result<Identity, Error> {
    match provider {
        "CrazyGames" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct CrazyClaims {
                // Expiration date/time. For example, 1670332280.
                // exp: usize,
                // For example, "20267".
                // game_id: String,
                // For example, 1670328680.
                // iat: usize,
                // For example, "https://images.crazygames.com/userportal/avatars/16.png".
                // profile_picture_url: String,
                // Unique. For example, "UOuZBKgjwpY9k4TSBB2NPugbsHD3".
                user_id: String,
                // Same as Discord username. For example, "RustyCake.ZU9H".
                username: String, // For example, "RustyCake.ZU9H", // same as Discord username
            }
            let public_key_pem = "-----BEGIN RSA PUBLIC KEY-----\nMIICCgKCAgEAxQ5jeskVJGg2y0JUo/iYBcqYcyud+xBKeTrSjdhvkprGMX7wtIUN\nrPRmrzJxbo8YkNSBPY2+l4HXTyi7hkDPPNtvMOuIiPkKg2+sXzqRcND5OnUwOH1b\nhzIETTAlZlQviTPYjlxWf4x9dYeVU/BemVW/s2EOjqj0/SVREBrNuWbFg28Er0Cx\nMu/UGKz6lV435Cdz+o9LIbnDPWOL2KsMJ6y+kwe1wBWSwnhiSmg6ZAyk79+N0l7L\nCAL668H3utG0aNY8/CIdup/xyrINSFXlqMpRD3Zq5fDYk5epy3cwCRpxyAkfBLor\nD4eHt7ybxT2e4nN8bjwi7ERyC9Znd5BSPW+Q9Za7pDi+9cr74etB08DVAP7woBO0\niZ3rrw0+CuZGg+WqmB85fzlnJHzTagMXej9O1lv11fcLCgglmpc6qjbfLIXgFEn5\nsMOmxLubzzqftYqEOXCxzU/y8w7EZcNi4ewsKFBizLLczcCgkZHuehmF/XanKlkj\nj59i63jjV1kB1Ps8QF59+rv9i4S6cP9ca1kNvaRDfdgtcfmRSz/KnRKe6MizQ3Pz\nKLJf5XIITtTCldWyh6ymPiYroibIguS75qwUEsNbP9WDFH3CB75FtbQK0NbhAvcm\nb0ppIUTgCXSCToA+UWDEuU819GbkuPI0cPD5/YrqJdLkSeaBZfYC0uECAwEAAQ==\n-----END RSA PUBLIC KEY-----";
            let CrazyClaims {
                user_id, username, ..
            } = decode_token(jw_token, public_key_pem, "RS256")?;
            Ok(Identity {
                login_id: AuthenticatedId(format!("crazygames/{user_id}")),
                user_name: Some(UserName(username)),
            })
        }
        _ => Err(Error::String(format!(
            "{provider}: cannot validate JWT identity for this provider"
        ))),
    }
}
