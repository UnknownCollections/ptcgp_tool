use log::info;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub(crate) mod binary;
pub(crate) mod commands;
pub(crate) mod crypto;
pub(crate) mod hash;
pub(crate) mod proto;
pub(crate) mod unity;
pub(crate) mod utils;
pub(crate) mod xapk;

use anyhow::Result;

const VERSION: &str = env!("VERSION");

fn main() -> Result<()> {
    info!("Pokemon TCG Pocket Tool - v{}", VERSION);
    commands::run()
}
