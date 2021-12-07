use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Vertex not found in tree: {}", err))]
    VertexNotFound { err: String },
    #[snafu(display("Tree in malformed state: {}", err))]
    TreeMalformed { err: String },
    #[snafu(display("Middleware error `{}`: {} ", source, err))]
    TreeUnavailable {
        source: Box<dyn std::error::Error>,
        err: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
