use nodekit::app::App;
use nodekit::service;
use nodekit::event::{Event, AppEvent};

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use esp_idf_svc::hal::prelude::*;
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use esp_idf_svc::eventloop::EspSystemEventLoop;
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use log::{info, warn};
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use crow_panel_advance_5::crow_panel::{init_lcd_panel, RgbDisplay};
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use crow_panel_advance_5::stc8h1k28::Stc8h1k28;
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use crow_panel_advance_5::gt911::{Gt911, GT911_ADDR};
#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use nodekit::service::rtc::{Bm8563, BM8563_ADDR};

use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::Terminal;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
fn getrandom_custom(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    unsafe {
        esp_idf_svc::sys::esp_fill_random(dest.as_mut_ptr() as *mut core::ffi::c_void, dest.len());
    }
    Ok(())
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
getrandom::register_custom_getrandom!(getrandom_custom);

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
fn main() -> color_eyre::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    
    info!("Starting nodekit esp...");
    
    // Initialize NVS (required for many services)
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    // Initialize system event loop (required for networking/LWIP)
    let sys_loop = EspSystemEventLoop::take()?;

    // Initialize networking (LWIP TCP/IP stack)
    unsafe {
        let err = esp_idf_svc::sys::esp_netif_init();
        if err != 0 {
            warn!("esp_netif_init failed: {}", err);
        } else {
            info!("esp_netif_init succeeded.");
        }
    }
    
    // Reconfigure Task Watchdog to be more lenient (30s) and add current task to it
    unsafe {
        let wdt_config = esp_idf_svc::sys::esp_task_wdt_config_t {
            timeout_ms: 30000,
            idle_core_mask: (1 << 0) | (1 << 1),
            trigger_panic: true,
        };
        esp_idf_svc::sys::esp_task_wdt_reconfigure(&wdt_config);
        esp_idf_svc::sys::esp_task_wdt_add(std::ptr::null_mut());
    }

    color_eyre::install()?;

    info!("Taking peripherals...");
    let peripherals = Peripherals::take()?;
    
    // Initialize I2C (Shared for STC, RTC, Touch)
    info!("Initializing I2C...");
    let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15,
        peripherals.pins.gpio16,
        &i2c_config,
    )?;

    // Reset STC8H1K28 microcontroller (GPIO1) before starting I2C
    info!("Resetting STC...");
    Stc8h1k28::hardware_reset(peripherals.pins.gpio1).map_err(|e| color_eyre::eyre::eyre!("Hardware reset error: {}", e))?;

    // Initialize Peripheral Controller (Backlight, Buzzer, Audio)
    info!("Initializing STC...");
    {
        let mut stc = Stc8h1k28::new(&mut i2c);
        stc.init().map_err(|e| color_eyre::eyre::eyre!("STC init error: {}", e))?;
        info!("Setting backlight...");
        stc.set_backlight(0x10).map_err(|e| color_eyre::eyre::eyre!("STC backlight error: {}", e))?; // Max brightness
        info!("Unmuting STC...");
        stc.unmute().map_err(|e| color_eyre::eyre::eyre!("STC unmute error: {}", e))?;
    }

    // Initialize LCD Panel
    info!("Initializing LCD Panel...");
    let panel = init_lcd_panel();
    
    // Give the hardware some time to stabilize
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let mut display = RgbDisplay::new(panel);

    info!("Cleaning display...");
    display.clear(Rgb565::BLACK).map_err(|e| color_eyre::eyre::eyre!("Display clear error: {:?}", e))?;

    info!("Initializing Terminal...");
    let config = EmbeddedBackendConfig {
        font_regular: mousefood::fonts::MONO_10X20,
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    info!("Registering eventfd VFS...");
    // Register eventfd VFS (required for Tokio's IO reactor on ESP-IDF)
    unsafe {
        let config = esp_idf_svc::sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
        };
        if esp_idf_svc::sys::esp_vfs_eventfd_register(&config) != 0 {
             warn!("Failed to register eventfd VFS");
        }
    }

    info!("Initializing Tokio runtime...");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(2)
        .build()
        .map_err(|e| color_eyre::eyre::eyre!("Tokio runtime build error: {} (raw OS error: {:?})", e, e.raw_os_error()))?;
    info!("Tokio runtime initialized successfully.");

    let _rt_guard = rt.enter();

    let url = option_env!("ALGOD_URL").unwrap_or("http://localhost:4001");
    let token = option_env!("ALGOD_TOKEN").unwrap_or("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    info!("Creating App...");
    let app = App::new(url, token, None, true);

    // Initial draw to show splash screen immediately
    terminal.draw(|frame| {
        let viewport = nodekit::ui::viewport::ViewportComponent::new(&app);
        frame.render_widget(&viewport, frame.area());
    })?;

    // Initialize RTC and sync system time
    info!("Initializing RTC...");
    app.events.send(AppEvent::LoadingMessage("Initializing RTC...".to_string()));
    {
        let mut rtc = Bm8563::new(&mut i2c, BM8563_ADDR);
        if let Err(e) = rtc.init() {
            warn!("RTC init failed: {}", e);
        } else {
            info!("Syncing system time from RTC...");
            if let Err(e) = rtc.sync_system_time() {
                warn!("Failed to sync system time from RTC: {}", e);
            } else {
                info!("System time synced from RTC successfully.");
            }
        }
    }
    
    // Reset watchdog before starting the app to give it a full timeout window
    unsafe { esp_idf_svc::sys::esp_task_wdt_reset(); }

    let modem = peripherals.modem;

    let main_fut = Box::pin(async move {
        info!("Starting background services...");
        
        // Spawn WiFi connection in a separate thread so it doesn't block the app loop
        let wifi_sender = app.events.get_sender();
        std::thread::spawn(move || {
            info!("Initializing WiFi (background)...");
            match service::wifi::connect_wifi(
                modem,
                sys_loop,
                nvs,
                Some(wifi_sender.clone()),
            ) {
                Ok(_wifi) => {
                    info!("WiFi connected (background).");
                    // Keep WiFi handle alive
                    loop { std::thread::sleep(std::time::Duration::from_secs(60)); }
                }
                Err(e) => {
                    warn!("WiFi connection failed (background): {}", e);
                    let _ = wifi_sender.send(Event::App(AppEvent::LoadingMessage(format!("WiFi Error: {}", e))));
                }
            }
        });

        service::metrics::spawn_metrics_loop(
            app.events.get_sender(),
            url.to_string(),
            token.to_string(),
        );
        service::node::spawn_node_loop(
            app.events.get_sender(),
            app.client.clone(),
            None,
        );
        app.events.send(AppEvent::LoadingMessage("Starting node loop...".to_string()));

        // Spawn touch polling task
        let sender = app.events.get_sender();
        std::thread::spawn(move || {
            let mut touch = Gt911::new(&mut i2c, GT911_ADDR);
            loop {
                if let Ok((true, touches)) = touch.read_status() {
                    if touches > 0 {
                        if let Ok(points) = touch.read_points(touches) {
                            for point in points {
                                let _ = sender.send(Event::Touch(point.x, point.y));
                                let mut stc = Stc8h1k28::new(touch.i2c_mut());
                                let _ = stc.beep(50);
                            }
                        }
                    } else {
                        let _ = touch.clear_status();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });

        app.events.send(AppEvent::LoadingMessage("Ready...".to_string()));
        app.run(terminal).await
    });

    rt.block_on(main_fut)?;

    Ok(())
}

#[cfg(not(any(target_arch = "xtensa", target_arch = "riscv32")))]
fn main() {
    panic!("This firmware is only intended for ESP32 (xtensa or riscv32) targets.");
}
