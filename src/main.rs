#[cfg(feature = "ssr")]
pub mod backend;

#[cfg(feature = "ssr")]
use crate::backend::run::run;

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub mod frontend;
mod macros;
mod structs;

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
use frontend::app::App;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    run().await
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(App);
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
fn main() {
    println!("Hello, world!");
}
