use tracing::info;

use crate::Error;

pub async fn send_heartbeat() -> Result<(), Error> {
    info!("Heartbeat");
    Ok(())
}
