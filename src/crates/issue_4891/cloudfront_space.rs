//! Test CloudFront with a URL including a space character

use async_trait::async_trait;
use reqwest::StatusCode;

use crate::test::{Test, TestResult};

use super::config::Config;
use super::request_url_and_expect_status;

/// The name of the test
const NAME: &str = "CloudFront with space";

/// Test CloudFront with a URL including a space character
///
/// This test request a URL with a space character from Cloudfront. The test expects the CDN to
/// return an HTTP 403 Forbidden response.
pub struct CloudfrontSpace<'a> {
    /// Configuration for this test
    config: &'a Config,
}

impl<'a> CloudfrontSpace<'a> {
    /// Create a new instance of the test
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl<'a> Test for CloudfrontSpace<'a> {
    async fn run(&self) -> TestResult {
        let url = format!(
            "{}/crates/{}/{}-{}.crate",
            self.config.cloudfront_url(),
            self.config.krate(),
            self.config.krate(),
            self.config.version()
        )
        .replace('+', " ");

        request_url_and_expect_status(NAME, &url, StatusCode::FORBIDDEN).await
    }
}

#[cfg(test)]
mod tests {
    use mockito::ServerGuard;

    use crate::test_utils::*;

    use super::*;

    const KRATE: &str = "rust-cratesio-4891";
    const VERSION: &str = "0.1.0+1";

    async fn setup() -> (ServerGuard, Config) {
        let server = mockito::Server::new_async().await;

        let config = Config::builder()
            .krate(KRATE.into())
            .version(VERSION.into())
            .cloudfront_url(server.url())
            .fastly_url(String::new())
            .build();

        (server, config)
    }

    #[tokio::test]
    async fn succeeds_with_http_403_response() {
        let (mut server, config) = setup().await;

        let encoded_version = VERSION.replace('+', "%20");

        let mock = server
            .mock(
                "GET",
                format!("/crates/{KRATE}/{KRATE}-{encoded_version}.crate").as_str(),
            )
            .with_status(403)
            .create();

        let result = CloudfrontSpace::new(&config).run().await;

        // Assert that the mock was called
        mock.assert();

        assert!(result.success());
    }

    #[tokio::test]
    async fn fails_with_other_http_responses() {
        let (mut server, config) = setup().await;

        let encoded_version = VERSION.replace('+', "%20");

        let mock = server
            .mock(
                "GET",
                format!("/crates/{KRATE}/{KRATE}-{encoded_version}.crate").as_str(),
            )
            .with_status(200)
            .create();

        let result = CloudfrontSpace::new(&config).run().await;

        // Assert that the mock was called
        mock.assert();

        assert!(!result.success());
    }

    #[test]
    fn trait_send() {
        assert_send::<CloudfrontSpace>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<CloudfrontSpace>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<CloudfrontSpace>();
    }
}
