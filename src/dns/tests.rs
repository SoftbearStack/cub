// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod dns_tests {
    use crate::common::CubConfig;
    use crate::dns::cloud_dns::{CloudDns, CloudDnsClient};
    use crate::dns::{AwsDns, DnsRecord, DnsRecordSet};
    use std::net::IpAddr;

    const AWS_DOMAIN: &str = "mazean.com";
    const LINODE_DOMAIN: &str = "zentakil.com";

    fn test_config() -> CubConfig {
        CubConfig::builder()
            .toml_str(
                r#"
            [aws]
            profile = "test_profile"
            [linode]
            personal_access_token = "TBD"
            "#,
            )
            .debug(true)
            .build()
            .expect("dns_test.toml")
    }

    #[tokio::test]
    async fn aws_dns_read_tests() {
        println!("Testing DNS read (for {AWS_DOMAIN})");
        let aws_dns = AwsDns::new(&test_config()).await;
        let _records = match aws_dns.read_dns_records(AWS_DOMAIN).await {
            Ok(records) => {
                print_records(&records);
                records
            }
            Err(e) => panic!("Cannot read DNS records: {e:?}"),
        };
    }

    #[tokio::test]
    async fn aws_dns_update_tests() {
        println!("Testing DNS update (for {AWS_DOMAIN}");
        let aws_dns = AwsDns::new(&test_config()).await;
        println!("Update text record for {AWS_DOMAIN}");
        let hostname1 = "test12345";
        let data1 = "Foo9876".to_string();
        match aws_dns
            .update_dns_metadata(AWS_DOMAIN, &hostname1, DnsRecord::Txt(data1), None)
            .await
        {
            Ok(result) => println!("Updated meta data: {result}"),
            Err(e) => panic!("Cannot update metadata: {e:?}"),
        }

        let hostname2 = "test12346";
        let data2 = "foo.softbear.com".to_string();
        match aws_dns
            .update_dns_route(AWS_DOMAIN, &hostname2, DnsRecord::Cname(data2), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }

        let hostname3 = "test12347";
        let ip_addr: IpAddr = "127.0.0.1".parse().expect("invalid IP addr");
        match aws_dns
            .update_dns_route(AWS_DOMAIN, &hostname3, DnsRecord::new_a(ip_addr), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }
    }

    #[tokio::test]
    async fn cloud_dns_tests() {
        let domain = AWS_DOMAIN;
        println!("Test DNS Cloud (with {domain})");
        let cloud_dns = CloudDnsClient::new(&test_config()).await;
        let _records = match cloud_dns
            .nameserver_api(Some("aws"))
            .await
            .read_dns_records(domain)
            .await
        {
            Ok(records) => {
                print_records(&records);
                records
            }
            Err(e) => panic!("Cannot read DNS records: {e:?}"),
        };

        println!("Update text record for {domain}");
        let hostname1 = "test23456";
        let data1 = "Foo8765".to_string();
        match cloud_dns
            .nameserver_api(Some("aws"))
            .await
            .update_dns_metadata(domain, &hostname1, DnsRecord::Txt(data1), None)
            .await
        {
            Ok(result) => println!("Updated meta data: {result}"),
            Err(e) => panic!("Cannot update metadata: {e:?}"),
        }

        let hostname2 = "test23457";
        let data2 = "foo.softbear.com".to_string();
        match cloud_dns
            .nameserver_api(Some("aws"))
            .await
            .update_dns_route(domain, &hostname2, DnsRecord::Cname(data2), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }

        let hostname3 = "test23458";
        let ip_addr: IpAddr = "127.0.0.1".parse().expect("invalid IP addr");
        match cloud_dns
            .nameserver_api(Some("aws"))
            .await
            .update_dns_route(domain, &hostname3, DnsRecord::new_a(ip_addr), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }
    }

    #[tokio::test]
    async fn linode_dns_tests() {
        let domain = LINODE_DOMAIN;
        println!("Test Linode DNS (for {domain})");
        let linode_dns = CloudDnsClient::new(&test_config()).await;
        let _records = match linode_dns
            .nameserver_api(Some("linode"))
            .await
            .read_dns_records(domain)
            .await
        {
            Ok(records) => {
                print_records(&records);
                records
            }
            Err(e) => panic!("Cannot read DNS records: {e:?}"),
        };

        println!("Update text record for {domain}");
        let hostname1 = "test12345";
        let data1 = "Foo9876".to_string();
        match linode_dns
            .nameserver_api(Some("linode"))
            .await
            .update_dns_metadata(domain, &hostname1, DnsRecord::Txt(data1), None)
            .await
        {
            Ok(result) => println!("Updated meta data: {result}"),
            Err(e) => panic!("Cannot update metadata: {e:?}"),
        }

        let hostname2 = "test12346";
        let data2 = "foo.softbear.com".to_string();
        match linode_dns
            .nameserver_api(Some("linode"))
            .await
            .update_dns_route(domain, &hostname2, DnsRecord::Cname(data2), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }

        let hostname3 = "test12347";
        let ip_addr: IpAddr = "127.0.0.1".parse().expect("invalid IP addr");
        match linode_dns
            .nameserver_api(Some("linode"))
            .await
            .update_dns_route(domain, &hostname3, DnsRecord::new_a(ip_addr), None)
            .await
        {
            Ok(result) => println!("Updated route: {result}"),
            Err(e) => panic!("Cannot update route: {e:?}"),
        }
    }

    fn print_records(record_set: &DnsRecordSet) {
        let metadata = record_set.metadata();
        println!("Read DNS metadata: {} records", metadata.len());
        let output: Vec<_> = metadata.iter().map(|(k, v)| format!("{k}={v:?}")).collect();
        println!("Metadata records:\n{}", output.join("\n"));
        let routes = record_set.routes();
        println!("Read DNS routes: {} records", routes.len());
        let output: Vec<_> = routes.iter().map(|(k, v)| format!("{k}={v:?}")).collect();
        println!("Route records:\n{}", output.join("\n"));
    }
}
