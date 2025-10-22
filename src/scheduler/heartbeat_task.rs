use tracing::info;

use crate::Error;

#[tracing::instrument(level = "info")]
pub async fn heartbeat() -> Result<(), Error> {
    info!("Heartbeat");
    Ok(())
}
