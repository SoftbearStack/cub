// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(test)]
mod jwt_tests {
    use crate::common::{CubConfig, Identity};
    use crate::jwt::{create_jwt, new_jwt_client, validate_jwt, validate_jwt_identity};
    use std::collections::HashMap;

    #[tokio::test]
    async fn jwt_identity_tests() {
        println!("JWT signing tests");
        let cub_config = CubConfig::builder()
            .toml_str(Default::default())
            .build()
            .expect("jwt_validation_tests.toml");

        let client = new_jwt_client(&cub_config);
        let jw_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJxYXRvb2xVc2VyMSIsImdhbWVJZCI6InFhdG9vbC1mYWtlLWdhbWUtaWQiLCJ1c2VybmFtZSI6IlVzZXIxIiwicHJvZmlsZVBpY3R1cmVVcmwiOiJodHRwczovL2ltYWdlcy5jcmF6eWdhbWVzLmNvbS91c2VycG9ydGFsL2F2YXRhcnMvMS5wbmciLCJpYXQiOjE3MjE1MjE2MjcsImV4cCI6MTcyMTUyNTIyN30.g6oNLi29GZI4B00IYcj90A4qCtds3M5YD_IKr3TM8n9iCjXoM7YrDCkDaPk3ZSmgYtMDigX9FVYwyDfMll_W_kFxVm3z3-4RXDUbN7yITVJKK8s4CgDBDnCC_XJpHhJMS2vMI0jKg5F7xF4rrLPLyD595fhq5SkD3tkmf8uRwhWACq9XpUO2ee0a0HtZyQk84zkSfKuYdINFK50_9_DvHhs681Abb8oSmstLIqj15VTwaV2TcL-38NP20BlsJpcToWb2IiET4JSGXvYyrTT0Fa-y1OdPQZ95ZF-mdvTw7YcN4eibB_oFRio7H20Ig9hoT4I340hY4VH1dyAsrXq_oHcyqok96hYTLy5tnZJYmyGNHkVtQ1tNhpjnyNScBellLmvRjek7_RipRXQaPDlQJVOsAeZJ0QB1Xg1M1M_KZS7_yGGM8dVhNiAjOTmK1Y-Jtv4fygFrQRNMgC6tIXcso_LdutoPnRSsBI8vS9gdZl2CkIjj_s_7j5R4TkkLHeXYXISzhY_4Zi7BN4tDtcd-Nn52W6yqsuayS4O167b7v-lsZq8E3f3XUazNwIfGVeoippAs268TnR3KBN7uxfZg8C6Hs4BFpMuOCvXUodcQtmenXxl_nD_JbJNWxgE5wqjMwEFm89aHNI4HXjXZGSFc4HywrypqwJMmUhS4G3VAazU";
        match validate_jwt_identity(&client, jw_token, "CrazyGames") {
            Ok(Identity {
                login_id,
                user_name,
            }) => println!("login_id={login_id}, user_name={user_name:?}",),
            Err(e) => {
                println!("cannot validate: {e:?}");
                Default::default()
            }
        };
    }

    #[tokio::test]
    async fn jwt_signing_tests() {
        println!("JWT signing tests");
        // The public/private key below were created for testing purposes via:
        //   openssl genrsa -out private_key.tmp 2048
        //   openssl pkey -in private_key.tmp -traditional > private_key.pem
        //   openssl rsa -in private_key.pem -pubout > public_key
        let cub_config = CubConfig::builder()
            .toml_str(
                r#"
                [jwt]
                private_key_pem = """
-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA2+TUX2E3jaEdmg1zorwAwLiA8LlwAKBffjsp5lZzVxZeVARC
wvRHmoCicp2c8e9DL4KrSAry8zJeCKlsZ4Kd1Mp//RQb/bP0V3tTpY3BARpPzfOH
sLH9RFEVQDvCP70teWjdQTam1LiJ4TYXZlKdEDpfcXrLjnu/HpHcb0+Z4tx2kct1
clsRHQhk06Def0QQjjWqd67ub4z3qV9Jhlv1LJ/skcI/uYhRf7R3VyBwDSvsEudg
RtTeVDH8Um7CXiiTDKe+Lp1tI/DIbSwuABhF7Dw7xdxshbhkryKZVLhTSSHE/bCR
B46DpJy9GUzNwqMoioct20eqMk1bklbfuBgrBwIDAQABAoIBAGCQEvDVpMslqvWp
HZQjgiMfgsPzcutbgcPRoFs9sIXYVVEI0/Z/xmfjQDMb4r1dh//3nlbTNBA3GJMu
L2QfOEcnK+BLseUN3umBx2BGqTBeSRhUbsxZxTH4d2APPgS2gx8zPSIzqTx101qa
Ydk1wzJKp/oR5gzqa6m1fPtGlfnIbLOk+cXEXaVQvJ1GliLzShVgw6Ix11dg8+is
+w62Kz4xKKlIZh6zXPcj1xurHK/4mL1IUP1+Yrw5uh3CVX44Wj8dDFjK2poMzKz4
gMtkB7FxuJWOctoAKe1yhgywOZBvhrnsE2MQGfMig4B8wGUye75fy7P2L1a2yFJg
iLR0Ta0CgYEA/7e+yiyANcmbHgCEjp+UPvUAsAgWwqZNfhr9YuDR9PgnWeU5pt63
Q2DmB3oIu4FMSqlgyrye9kC67qc7Z/5XpiKOfXyMCVoQRClYuYG9aPpO1MJAK0WH
wpJ8ToDtmQSkdj0Hr8BR4c17zkpnudhRCepLSdlVRtbNJLyWbomUkwUCgYEA3CL2
R88NtiRqqqIj/43WjFkBzdA7eT+J1hix+B6dRc+xhqFemoay+XhVhaOZz/8NAM9h
RCnk7CPhqyCr29kijFyQbUHyQwzypunzmHd/jz7ZezZyPlsVpC76ho6Mj6UIb7Nw
Tt7fr5g+hGLA3cwYjN/iIx8Q+wWW98VYhL4NO5sCgYAGaylJz9YkA3x2Q1MQdWb2
MZYj1QAlQKFfUfQcQEJk4Lm0IvHQg3ScJ1l+xIxlkHhGw3ufex6OVc+bX+04zgSL
MgDbm320WmNgIp2MgnoroWTLKFkN/P/MXXrrSYctORWbtip0OeKURWEfK3TxEEHw
esYLA36FeazKiEVKXv+wtQKBgDyhHHeWnU4nJYGteoCuDgNFmGuZCGhSiaH/1zRh
KivKEjjkROwGYVC4RcWy03An7OrmMwHVEAnBsCuzqeG5IfzKmbSdzx2MeWBjWwYJ
E4beZoO68Sgfagx4K+PXavs9Ft+86heu5qi0I7POhxQPXEugdeX6bnDUj0nafpDA
z2A1AoGBAOpZFE8dhHvE6V0XlKpDbGdD+cLDj/+DP3xWkT3iTM3Zy0Lr0hHrsLYH
+9z06WmsIRL1w9GBsVOZKGXgFa0QwzVeEo24tirp4Z4+ecSfPP+i0rBtlPkHkCzQ
eXH4eQz6Vd2VLDotVnL32XNeql70NkJZaLP+kJdDiDx1ciGgcGp7
-----END RSA PRIVATE KEY-----
"""
                public_key_pems = { "default" = """-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA2+TUX2E3jaEdmg1zorwA
wLiA8LlwAKBffjsp5lZzVxZeVARCwvRHmoCicp2c8e9DL4KrSAry8zJeCKlsZ4Kd
1Mp//RQb/bP0V3tTpY3BARpPzfOHsLH9RFEVQDvCP70teWjdQTam1LiJ4TYXZlKd
EDpfcXrLjnu/HpHcb0+Z4tx2kct1clsRHQhk06Def0QQjjWqd67ub4z3qV9Jhlv1
LJ/skcI/uYhRf7R3VyBwDSvsEudgRtTeVDH8Um7CXiiTDKe+Lp1tI/DIbSwuABhF
7Dw7xdxshbhkryKZVLhTSSHE/bCRB46DpJy9GUzNwqMoioct20eqMk1bklbfuBgr
BwIDAQAB
-----END PUBLIC KEY-----""" }
                "#,
            )
            .build()
            .expect("jwt_signing_tests.toml");
        let client = new_jwt_client(&cub_config);
        let claims_in: HashMap<String, String> = vec![("Foo", "1"), ("Bar", "2")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        let jwt = match create_jwt(&client, claims_in, 3600) {
            Ok(jwt) => jwt,
            Err(e) => panic!("cannot create JWT: {e:?}"),
        };
        println!("JWT is: {jwt}");

        let claims_out: HashMap<String, String> =
            validate_jwt(&client, &jwt, None).expect("cannot validate JWT");
        println!("{claims_out:?}");
    }
}
