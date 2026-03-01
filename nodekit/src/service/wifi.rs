use esp_idf_svc::wifi::{BlockingWifi, EspWifi, Configuration, ClientConfiguration};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use log::{info, warn};

use crate::event::{Event, AppEvent};
use tokio::sync::mpsc::UnboundedSender;

/// Connects to WiFi using SSID and Password provided during build via 
/// environment variables `WIFI_SSID` and `WIFI_PASS`.
pub fn connect_wifi(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    sender: Option<UnboundedSender<Event>>,
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

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().map_err(|_| color_eyre::eyre::eyre!("SSID too long"))?,
        password: pass.try_into().map_err(|_| color_eyre::eyre::eyre!("Password too long"))?,
        ..Default::default()
    }))?;

    if let Some(s) = &sender {
        let _ = s.send(Event::App(AppEvent::LoadingMessage("Starting WiFi...".to_string())));
    }
    wifi.start()?;
    info!("WiFi started.");
    
    // Reset watchdog before connection which can take some time
    unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

    let mut last_err = None;
    for attempt in 1..=3 {
        let msg = format!("Connecting to WiFi: {} (Attempt {}/3)...", ssid, attempt);
        info!("{}", msg);
        if let Some(s) = &sender {
            let _ = s.send(Event::App(AppEvent::LoadingMessage(msg)));
        }

        match wifi.connect() {
            Ok(_) => {
                info!("WiFi connected.");
                // Reset watchdog after connection
                unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

                if let Some(s) = &sender {
                    let _ = s.send(Event::App(AppEvent::LoadingMessage("Waiting for IP...".to_string())));
                }

                match wifi.wait_netif_up() {
                    Ok(_) => {
                        info!("WiFi netif up.");
                        // Reset watchdog after netif is up
                        unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

                        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
                        info!("WiFi IP info: {:?}", ip_info);
                        
                        if let Some(s) = &sender {
                            let _ = s.send(Event::App(AppEvent::LoadingMessage(format!("Connected: {:?}", ip_info.ip))));
                            let _ = s.send(Event::App(AppEvent::WifiConnected));
                        }

                        return Ok(wifi);
                    }
                    Err(e) => {
                        warn!("WiFi wait_netif_up failed (attempt {}): {}", attempt, e);
                        last_err = Some(e.into());
                    }
                }
            }
            Err(e) => {
                warn!("WiFi connect failed (attempt {}): {}", attempt, e);
                last_err = Some(e.into());
            }
        }
        
        if attempt < 3 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    Err(last_err.unwrap_or_else(|| color_eyre::eyre::eyre!("WiFi connection failed after 3 attempts")))
}
