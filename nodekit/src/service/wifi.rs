use esp_idf_svc::wifi::{BlockingWifi, EspWifi, Configuration, ClientConfiguration};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use log::info;

/// Connects to WiFi using SSID and Password provided during build via 
/// environment variables `WIFI_SSID` and `WIFI_PASS`.
pub fn connect_wifi(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> color_eyre::Result<BlockingWifi<EspWifi<'static>>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let ssid = option_env!("WIFI_SSID").unwrap_or("");
    let pass = option_env!("WIFI_PASS").unwrap_or("");

    if ssid.is_empty() {
        return Err(color_eyre::eyre::eyre!("WiFi configuration missing. Please set WIFI_SSID and WIFI_PASS environment variables during build."));
    }

    info!("Connecting to WiFi: {}...", ssid);

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().map_err(|_| color_eyre::eyre::eyre!("SSID too long"))?,
        password: pass.try_into().map_err(|_| color_eyre::eyre::eyre!("Password too long"))?,
        ..Default::default()
    }))?;

    wifi.start()?;
    info!("WiFi started.");
    
    // Reset watchdog before connection which can take some time
    unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

    wifi.connect()?;
    info!("WiFi connected.");

    // Reset watchdog after connection
    unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

    wifi.wait_netif_up()?;
    info!("WiFi netif up.");

    // Reset watchdog after netif is up
    unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi IP info: {:?}", ip_info);

    Ok(wifi)
}
