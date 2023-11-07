use std::{
    fs, io,
    path::{self, Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

pub struct ServerResponder {
    /// The directory to host, if any
    host_directory: Option<PathBuf>,
    /// An override on the returned status code.
    status_code: Option<u16>,
    /// An extra delay to add before responding
    extra_delay: Duration,
}

impl ServerResponder {
    #[must_use]
    pub const fn builder() -> ServerResponderBuilder {
        ServerResponderBuilder::new()
    }

    #[must_use]
    pub fn respond(&self, request: &tiny_http::Request) -> tiny_http::ResponseBox {
        let starting_time = Instant::now();
        let mut response = if let Some(directory) = &self.host_directory {
            /// Respond to the given request
            fn fetch_file(
                host_directory: &Path,
                request: &tiny_http::Request,
            ) -> tiny_http::ResponseBox {
                // The requested URL begins with a `/`, which we need to drop for `Path::join`
                // to append the requested path to the directory we're hosting.
                let Some(requested_file) = request.url().get(1..) else {
                    return tiny_http::Response::empty(404).boxed();
                };
                // Check for trying to escape `out_dir` and respond with a 400
                if Path::new(requested_file).components().any(|s| {
                    matches!(
                        s,
                        path::Component::ParentDir
                            | path::Component::RootDir
                            | path::Component::Prefix(_)
                    )
                }) {
                    return tiny_http::Response::empty(400).boxed();
                }
                // Return the file
                match fs::File::open(host_directory.join(requested_file)) {
                    Ok(requested_file) => tiny_http::Response::from_file(requested_file).boxed(),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        tiny_http::Response::empty(404).boxed()
                    }
                    Err(e) => tiny_http::Response::from_string(e.to_string())
                        .with_status_code(500)
                        .boxed(),
                }
            }
            fetch_file(directory, request)
        } else {
            tiny_http::Response::empty(200).boxed()
        };
        if let Some(status_code) = self.status_code {
            response = response.with_status_code(status_code);
        }
        thread::sleep((starting_time + self.extra_delay).saturating_duration_since(Instant::now()));
        response
    }
}

pub struct ServerResponderBuilder {
    /// The directory to host, if any
    host_directory: Option<PathBuf>,
    /// An override on the returned status code.
    status_code: Option<u16>,
    /// An extra delay to add before responding
    extra_delay: Option<Duration>,
}
impl ServerResponderBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            host_directory: None,
            status_code: None,
            extra_delay: None,
        }
    }

    #[must_use]
    pub fn host_directory(mut self, dir: PathBuf) -> Self {
        if self.host_directory.replace(dir).is_some() {
            panic!("Set host directory multiple times");
        }
        self
    }

    #[must_use]
    pub fn status_code(mut self, code: u16) -> Self {
        if self.status_code.replace(code).is_some() {
            panic!("Set status code override multiple times");
        }
        self
    }

    #[must_use]
    pub fn extra_delay(mut self, delay: Duration) -> Self {
        if self.extra_delay.replace(delay).is_some() {
            panic!("Set extra delay multiple times");
        }
        self
    }

    #[must_use]
    pub fn build(self) -> ServerResponder {
        ServerResponder {
            host_directory: self.host_directory,
            status_code: self.status_code,
            extra_delay: self.extra_delay.unwrap_or(Duration::ZERO),
        }
    }
}
