// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::{CloudVideos, VideoRecord, VideoResourceId};
use crate::common::{CubConfig, Error};
use crate::log::StringLogger;
use async_trait::async_trait;
use hyper::StatusCode;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const EMBED_HTML_PREFIX: &str = "https://www.youtube.com/embed/";
const VIDEO_URL_PREFIX: &str = "https://www.youtube.com/watch?v=";
const YOUTUBE_RESOURCE_PREFIX: &str = "youtube";

/// Youtube cloud.
pub struct YoutubeVideos {
    api_key: String,
    client: Client,
    debug: bool,
}

impl YoutubeVideos {
    const TIMEOUT_SECS: u64 = 5;

    /// Create a `CloudVideo` for Youtube.
    pub fn new(cub_config: &CubConfig) -> Self {
        #[derive(Deserialize)]
        struct YoutubeConfig {
            api_key: String,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            youtube: YoutubeConfig,
        }
        let ConfigToml {
            youtube: YoutubeConfig { api_key },
        } = cub_config.get().expect("youtube.toml");

        Self {
            api_key,
            client: Client::builder()
                .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
                .http1_only()
                .build()
                .unwrap(),
            debug: cub_config.debug(),
        }
    }

    fn map_error(e: reqwest::Error) -> Error {
        Error::Http(StatusCode::FAILED_DEPENDENCY, format!("{}", e))
    }

    fn parse_resource_id(resource_id: &VideoResourceId) -> Result<String, Error> {
        let mut split = resource_id.0.splitn(2, '/');
        if split
            .next()
            .map(|s| s != YOUTUBE_RESOURCE_PREFIX)
            .unwrap_or(true)
        {
            Err(Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!(
                    "{}: expected '{YOUTUBE_RESOURCE_PREFIX}' prefix in resource ID",
                    resource_id.0
                ),
            ))
        } else if let Some(id) = split.next() {
            Ok(id.to_string())
        } else {
            Err(Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!("{}: invalid video resource ID", resource_id.0),
            ))
        }
    }

    fn parse_result<'a, T: Deserialize<'a>>(text: &'a String) -> Result<T, Error> {
        match serde_json::from_str(&text) {
            Ok(response) => Ok(response),
            Err(_) => {
                #[derive(Deserialize)]
                struct YoutubeReason {
                    // code: usize,
                    message: String,
                }
                #[derive(Deserialize)]
                struct YoutubeError {
                    error: YoutubeReason,
                }
                match serde_json::from_str(&text) {
                    Ok(YoutubeError {
                        error: YoutubeReason { message },
                    }) => Err(Error::Http(
                        StatusCode::NOT_FOUND,
                        format!("youtube error: {message}"),
                    )),
                    Err(_) => Err(Error::Http(
                        StatusCode::FAILED_DEPENDENCY,
                        format!("cannot parse youtube error: {text}"),
                    )),
                }
            }
        }
    }
}

#[async_trait]
impl CloudVideos for YoutubeVideos {
    fn embeddable_html(
        &self,
        VideoRecord { video_url, .. }: &VideoRecord,
    ) -> Result<String, Error> {
        if !video_url.starts_with(VIDEO_URL_PREFIX) {
            Err(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("{video_url}: not a Youtube video URL"),
            ))
        } else {
            let video_id = &video_url[VIDEO_URL_PREFIX.len()..];
            Ok(format!(
                r#"
                <iframe
                  allow='accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture'
                  allowfullscreen
                  frameborder='0'
                  height='135'
                  src='{EMBED_HTML_PREFIX}{video_id}'
                  width='240'
                  />
                "#
            )
            .trim()
            .split(' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "))
        }
    }

    async fn list_playlist(
        &self,
        id: &VideoResourceId,
    ) -> Result<Vec<(VideoResourceId, VideoRecord)>, Error> {
        let _logger = StringLogger::new(self.debug);
        let playlist_id = Self::parse_resource_id(id)?;
        let parameters: Vec<_> = vec![
            ("part", "snippet"),
            ("key", &self.api_key),
            ("playlistId", &playlist_id),
        ]
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
        let query = parameters.join("&");
        let url = format!("https://www.googleapis.com/youtube/v3/playlistItems?{query}");
        //      if self.debug {
        //          logger.trace(format!("url={url}"));
        //      }
        let request = self.client.get(&url).build().map_err(Self::map_error)?;

        let response = self
            .client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        let result = response.text().await.map_err(Self::map_error)?;

        let response: YoutubeResponse = Self::parse_result(&result)?;
        Ok(response
            .items
            .into_iter()
            .map(
                |YoutubeItem {
                     snippet:
                         YoutubeSnippet {
                             resource_id: YoutubeResourceId { video_id },
                             thumbnails,
                             title,
                         },
                 }| {
                    (
                        VideoResourceId(format!("{YOUTUBE_RESOURCE_PREFIX}/{url}")),
                        VideoRecord {
                            caption: title,
                            teaser_url: thumbnails
                                .get("default")
                                .map(|YoutubeThumbnail { url, .. }| url.to_string())
                                .unwrap_or(String::default()),
                            video_url: format!("{VIDEO_URL_PREFIX}{video_id}"),
                        },
                    )
                },
            )
            .collect())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoutubeItem {
    snippet: YoutubeSnippet,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoutubeResourceId {
    video_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoutubeResponse {
    items: Vec<YoutubeItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoutubeSnippet {
    resource_id: YoutubeResourceId,
    thumbnails: HashMap<String, YoutubeThumbnail>,
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YoutubeThumbnail {
    // height: usize,
    // width: usize,
    url: String,
}
