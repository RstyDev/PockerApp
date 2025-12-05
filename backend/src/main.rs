#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
extern crate tokio;

use crate::run::run;

mod run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run().await
}
