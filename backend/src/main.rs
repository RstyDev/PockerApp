extern crate tokio;

use crate::run::run;

mod run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run().await
}
