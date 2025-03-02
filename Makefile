# SPDX-FileCopyrightText: 2024 Softbear, Inc.
# SPDX-License-Identifier: LGPL-3.0-or-later

.PHONY: all doc rustup test

all:
	cargo build

doc:
	cargo doc

rustup:
	rustup override set stable

test:
	cargo test -- --nocapture

test_dns:
	cargo test aws_dns_read_tests -- --nocapture
	# cargo test aws_dns_update_tests -- --nocapture
	# cargo test cloud_dns_tests -- --nocapture
	# cargo test linode_dns_tests -- --nocapture

test_hosts:
	cargo test linode_host_tests -- --nocapture

test_jwt:
	# cargo test jwt_identity_tests -- --nocapture
	cargo test jwt_signing_tests -- --nocapture

test_logger:
	cargo test logger_tests -- --nocapture

test_markdown:
	cargo test --features yew_markdown yew_markdown_tests -- --nocapture

test_stripe:
	cargo test customer_tests -- --nocapture
	#cargo test product_tests -- --nocapture
	#cargo test price_tests -- --nocapture

test_time:
	cargo test chrono_tests -- --nocapture
	cargo test time_casts_01 -- --nocapture
	cargo test time_tests_now -- --nocapture

test_translate:
	cargo test translate_tests -- --nocapture

test_videos:
	cargo test cloud_video_tests -- --nocapture