// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(test)]
mod videos_test {
    use crate::common::CubConfig;
    use crate::videos::{CloudVideos, VideoResourceId, YoutubeVideos};

    #[tokio::test]
    async fn cloud_video_tests() {
        println!("youtube_video_tests");
        let secrets_toml = r#"
            [youtube]
            api_key = "TBD"
        "#;
        if secrets_toml.len() < 32 {
            panic!("secrets_toml must be edited or this test will fail");
        }
        let cub_config = CubConfig::builder()
            .toml_str(secrets_toml)
            .debug(true)
            .build()
            .expect("cloud_video_tests.toml");
        let youtube_videos = YoutubeVideos::new(&cub_config);

        let playlist_id = VideoResourceId("youtube/PLGmdBhbsCAZVrzREncOQ0EShJXrZuiXc7".to_string());
        match youtube_videos.list_playlist(&playlist_id).await {
            Ok(list) => println!("succeeded {list:?}"),
            Err(e) => println!("{e:?}"),
        }
    }
}
