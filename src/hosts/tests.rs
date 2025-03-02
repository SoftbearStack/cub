// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(test)]
mod hosts_test {
    use crate::common::CubConfig;
    use crate::hosts::{CloudHosts, LinodeHosts};

    #[tokio::test]
    async fn linode_host_tests() {
        println!("linode_host_tests");
        let secrets_toml = r#"
            [linode]
            authorized_ssh_key = "ssh-rsa 1234"
            firewall_ids = { "default" = "linode/12345" }
            personal_access_token = "1234"
        "#;
        if secrets_toml.len() < 130 {
            panic!("secrets_toml must be edited or this test will fail");
        }
        let cub_config = CubConfig::builder()
            .toml_str(secrets_toml)
            .debug(true)
            .build()
            .expect("linode_host_tests.toml");
        let linode_host = LinodeHosts::new(&cub_config);
        // println!("{:?}", linode_host.list_scripts().await);

        let datacenter_list = linode_host.list_datacenters().await.expect("datacenters");
        if datacenter_list.is_empty() {
            panic!("cannot list datacenters");
        }
        println!("datacenter list: {datacenter_list:?}");

        const SCRIPT_TEST: bool = false;
        if SCRIPT_TEST {
            let id = linode_host
                .create_script("candywrapper_script01", "#!/bin/sh\necho hello world")
                .await
                .expect("create_script");
            println!("create script succeeded.  id={id:?}");
            linode_host.delete_script(&id).await.expect("delete_script");
            println!("delete script succeeded.");
        }
        const HOST_TEST: bool = true;
        if HOST_TEST {
            let datacenter = datacenter_list[0].clone();
            let label = "candywrapper_test01";
            let hostname = label;
            let script = "#!/bin/sh\napt update\napt install -y git";
            match linode_host
                .create_host(label, None, hostname, datacenter, script, None)
                .await
            {
                Ok((id, addr)) => {
                    println!("create host succeeded.  id={id:?}, addr={addr}");
                    linode_host.delete_host(&id).await.expect("delete_host");
                    println!("delete host succeeded.");
                }
                Err(e) => println!("create host failed: {e:?}"),
            }
        }
        const LIST_TEST: bool = false;
        if LIST_TEST {
            println!("list hosts");
            for host in match linode_host.list_hosts().await {
                Ok(hosts) => hosts,
                Err(e) => {
                    println!("list failed: {e:?}");
                    vec![]
                }
            } {
                println!("{host:?}");
            }
        }
    }
}
