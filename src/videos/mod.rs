// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

/// Video cloud trait
mod cloud_videos;

/// Support for Youtube.
mod youtube;

/// Unit tests
mod tests;

pub use self::cloud_videos::{CloudVideos, VideoRecord, VideoResourceId};
pub use self::youtube::YoutubeVideos;
