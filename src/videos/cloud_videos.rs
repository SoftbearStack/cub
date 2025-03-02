// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::common::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Video resource ID. For example, the ID of a playlist or video.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VideoResourceId(pub String);
crate::impl_wrapper_str!(VideoResourceId);

/// Video cloud.
#[async_trait]
pub trait CloudVideos {
    /// Return embeddable HTML.
    fn embeddable_html(&self, id: &VideoRecord) -> Result<String, Error>;

    /// List the video records in a playlist.
    async fn list_playlist(
        &self,
        id: &VideoResourceId,
    ) -> Result<Vec<(VideoResourceId, VideoRecord)>, Error>;
}

/// Video record.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct VideoRecord {
    /// Video caption.
    pub caption: String,
    /// URL of teaser image.
    pub teaser_url: String,
    /// URL of video.
    pub video_url: String,
}

impl VideoRecord {
    /// Build caption.
    pub fn caption(mut self, value: String) -> Self {
        self.caption = value;
        self
    }

    /// Build teaser URL.
    pub fn teaser_url(mut self, value: String) -> Self {
        self.teaser_url = value;
        self
    }

    /// Build video URL.
    pub fn video_url(mut self, value: String) -> Self {
        self.video_url = value;
        self
    }
}
