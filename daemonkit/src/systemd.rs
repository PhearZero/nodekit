use zbus::{Connection, Proxy};
use anyhow::Result;

pub async fn start_service() -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    ).await?;
    
    // StartUnit(name, mode)
    let _: (zbus::zvariant::OwnedObjectPath,) = proxy.call("StartUnit", &("algorand.service", "replace")).await?;
    Ok(())
}

pub async fn stop_service() -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    ).await?;
    
    // StopUnit(name, mode)
    let _: (zbus::zvariant::OwnedObjectPath,) = proxy.call("StopUnit", &("algorand.service", "replace")).await?;
    Ok(())
}
