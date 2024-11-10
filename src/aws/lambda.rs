// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::body::{to_bytes, Body};
use axum::http::StatusCode;
use axum::Router;
use base64::{alphabet, engine, Engine};
use core::convert::TryFrom;
use core::future::Future;
use core::task::Context;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Method, Request};
use lambda_runtime::{Error, LambdaEvent, Service};
use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env::var;
use std::pin::Pin;
use std::str::FromStr;
use urlencoding::encode;

const DEBUG1: bool = false;
const DEBUG2: bool = false;

/// Returns true when executable is run in AWS Lambda environment.
pub fn is_lambda_env() -> bool {
    var("AWS_LAMBDA_RUNTIME_API").is_ok()
}

/// Run a router on a Lambda Proxy invoked via AWS API Gateway. The
/// AWS API Gateway binary media type must be set to `*/*` so that binary
/// data will be encoded using base 64.
pub async fn run_router_on_lambda(router: Router) -> Result<(), Error> {
    println!("Begin running router on lambda");
    lambda_runtime::run(RouterWrapper(router)).await?;
    println!("Done running router on lambda");
    Ok(())
}

/// The `GwRequest` (gateway request) type parses JSON from AWS API Gateway into an `ApiGatewayEvent` struct.
type GwRequest = Request<Body>;

/// The `RouterWrapper` struct layers additional functionality on top of `axum::Router` to parse JSON
/// requests from AWS API Gateway, and provide JSON responses to AWS API Gateway.
struct RouterWrapper(Router);

impl Service<LambdaEvent<ApiGatewayEvent>> for RouterWrapper {
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<serde_json::Value, Infallible>>>>;
    type Response = serde_json::Value;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        <Router as lambda_runtime::Service<GwRequest>>::poll_ready(&mut self.0, cx)
    }

    fn call(&mut self, lambda_event: LambdaEvent<ApiGatewayEvent>) -> Self::Future {
        let path = lambda_event.payload.path.clone();
        if DEBUG1 {
            println!(
                "lambda begins with path {}",
                path.clone().unwrap_or_default()
            );
        }
        let request = GwRequest::try_from(lambda_event.payload);
        let router_result = request.map(|r| self.0.call(r));
        let fut = async move {
            match router_result {
                Ok(method_result) => {
                    match method_result.await {
                        Ok(result) => {
                            let (parts, body) = result.into_parts();
                            let mut headers = serde_json::Map::new();
                            for (k, v) in &parts.headers {
                                if let Ok(value_str) = v.to_str() {
                                    headers.insert(k.as_str().to_string(), json!(value_str));
                                }
                            }
                            // The following should match the binary media types in API Gateway settings.
                            let binary =
                                match headers.get("content-type").map(|v| v.as_str()).flatten() {
                                    Some("application/octet-stream") => true,
                                    Some("image/gif") => true,
                                    Some("image/jpg") => true,
                                    Some("image/jpeg") => true,
                                    Some("image/png") => true,
                                    Some("image/webp") => true,
                                    _ => false,
                                };

                            match to_bytes(body, usize::MAX).await {
                                Ok(body) => {
                                    if DEBUG2
                                        && !StatusCode::is_success(&parts.status)
                                        && parts.status != StatusCode::SEE_OTHER
                                    {
                                        // Normally the body of errors is hidden, so return OK even for errors.
                                        Ok(json!({
                                            "body": format!("{{ \"error\": \"{}\", \"message\": \"{}\", \"path\": \"{}\", \"status\": {} }}", parts.status.canonical_reason().unwrap_or_default(), String::from_utf8_lossy(&body), path.unwrap_or_default(), parts.status.as_u16()),
                                            "headers": { "content-type": "application/json"},
                                            "statusCode": StatusCode::OK.as_u16(),
                                        }))
                                    } else {
                                        let encoded_body = if binary {
                                            println!(
                                                "Downloading a binary file of length {}",
                                                body.len()
                                            );
                                            let engine = engine::GeneralPurpose::new(
                                                &alphabet::STANDARD,
                                                engine::general_purpose::PAD,
                                            );
                                            engine.encode(&body)
                                        } else {
                                            String::from_utf8_lossy(&body).into()
                                        };
                                        if DEBUG1 || !StatusCode::is_success(&parts.status) {
                                            println!(
                                                "lambda {} ends with status {}: {}",
                                                path.clone().unwrap_or_default(),
                                                parts.status.as_u16(),
                                                &encoded_body
                                            );
                                        }
                                        Ok(json!({
                                            "body": encoded_body,
                                            "headers": headers,
                                            "isBase64Encoded": binary,
                                            "statusCode": parts.status.as_u16(),
                                        }))
                                    }
                                }
                                Err(e) => {
                                    // In practice, this never happens.
                                    println!("Teapot error {:?}", e);
                                    Ok(json!({
                                        "body": "Result body error",
                                        "headers": { "content-type": "application/json"},
                                        "statusCode": StatusCode::IM_A_TEAPOT.as_u16(),
                                    }))
                                }
                            }
                        }
                        Err(e) => {
                            println!("Method error {:?}", e);
                            // For example, if a GET is performed on a path that only supports POST.
                            Ok(json!({
                                "body": "Method error",
                                "headers": { "content-type": "application/json"},
                                "statusCode": StatusCode::METHOD_NOT_ALLOWED.as_u16(),
                            }))
                        }
                    }
                }
                Err(e) => {
                    // In practice, this never happens (even if path is not found).
                    println!("Router error {:?}", e);
                    Ok(json!({
                        "body": "Router error",
                        "headers": { "content-type": "application/json"},
                        "statusCode": StatusCode::NOT_FOUND.as_u16(),
                    }))
                }
            }
        };
        Box::pin(fut)
    }
}

/// Convert an AWS API Gateway event into a `hyper::Request` suitable for `axum::Router`.
impl TryFrom<ApiGatewayEvent> for GwRequest {
    type Error = Error;

    fn try_from(gw_event: ApiGatewayEvent) -> Result<Self, Self::Error> {
        let method = Method::try_from(gw_event.http_method.unwrap_or("GET".to_string()).as_str())?;

        let builder =
            if let Some(ApiGatewayRequestContext::WebSocket(context)) = gw_event.request_context {
                let ApiGatewayV2WebsocketContext {
                    connection_id,
                    event_type,
                } = context;
                let path = format!("/ws/{event_type:?}/{connection_id}");
                let uri = append_query_string(&path, &gw_event.multi_value_query_string_parameters);
                Request::builder().method("POST").uri(uri)
            } else {
                let path = gw_event.path.unwrap_or("/".to_string());
                let uri = append_query_string(&path, &gw_event.multi_value_query_string_parameters);
                let mut builder = Request::builder().method(method).uri(uri);

                if let (Some(headers_mut), Some(multi_value_headers)) =
                    (builder.headers_mut(), gw_event.multi_value_headers)
                {
                    // For example:
                    //   host: "abcdefghij.execute-api.us-east-1.amazonaws.com"
                    //   x-forwarded-for: "1.2.3.4, 5.6.7.8"
                    // Plus accept-encoding, cache-control, cookie, user-agent, etc.
                    let headers = multi_value_headers.iter().flat_map(|(key, value_list)| {
                        value_list
                            .iter()
                            .map(move |value| (key.as_str(), value.as_str()))
                    });
                    for (key, value) in headers {
                        if let (Ok(key), Ok(value)) =
                            (HeaderName::from_str(key), HeaderValue::from_str(value))
                        {
                            headers_mut.insert(key, value);
                        }
                    }
                }
                builder
            };

        let body = if gw_event.is_base64_encoded {
            let engine =
                engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::PAD);
            engine.decode::<Vec<u8>>(gw_event.body.unwrap_or_default().into())?
        } else {
            gw_event.body.unwrap_or_default().into()
        };

        let request = builder.body(Body::from(body))?;

        Ok(request)
    }
}

/// An AWS API Gateway REST or web socket event, with only those fields necessary for `RouterWrapper`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiGatewayEvent {
    body: Option<String>,
    http_method: Option<String>,
    #[serde(default)]
    is_base64_encoded: bool,
    multi_value_headers: Option<HashMap<String, Vec<String>>>,
    multi_value_query_string_parameters: Option<HashMap<String, Vec<String>>>,
    path: Option<String>,
    request_context: Option<ApiGatewayRequestContext>,
}

fn append_query_string(
    path: &str,
    multi_value_query_string_parameters: &Option<HashMap<String, Vec<String>>>,
) -> String {
    if let Some(query_parms) = multi_value_query_string_parameters {
        let query = query_parms
            .iter()
            .flat_map(|(k, vec)| vec.iter().map(move |v| format!("{}={}", &k, encode(&v))))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{}", &path, &query)
    } else {
        path.to_owned()
    }
}

/// REST or web socket request context.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum ApiGatewayRequestContext {
    WebSocket(ApiGatewayV2WebsocketContext),
    Rest(IgnoredAny),
}

/// The context for an AWS API Gateway v2 socket event, with only the necessary fields.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2WebsocketContext {
    connection_id: String,
    event_type: WebsocketEventType,
}

/// Web socket event types.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum WebsocketEventType {
    Connect,
    Disconnect,
    Message,
}
