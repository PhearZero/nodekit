use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::sys::{timeval, settimeofday, timezone};
use color_eyre::eyre::OptionExt;

pub const BM8563_ADDR: u8 = 0x51;

#[derive(Debug, Clone, Copy, Default)]
pub struct DateTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day: u8,
    pub weekday: u8,
    pub month: u8,
    pub year: u16,
}

pub struct Bm8563<'a, 'b> {
    i2c: &'a mut I2cDriver<'b>,
    addr: u8,
}

impl<'a, 'b> Bm8563<'a, 'b> {
    pub fn new(i2c: &'a mut I2cDriver<'b>, addr: u8) -> Self {
        Self { i2c, addr }
    }

    pub fn init(&mut self) -> Result<(), esp_idf_svc::sys::esp_err_t> {
        self.write_reg(0x00, 0x00)?;
        self.write_reg(0x01, 0x00)?;
        Ok(())
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), esp_idf_svc::sys::esp_err_t> {
        let data = [reg, val];
        self.i2c.write(self.addr, &data, 1000).map_err(|e| e.code())?;
        Ok(())
    }

    fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), esp_idf_svc::sys::esp_err_t> {
        self.i2c.write_read(self.addr, &[reg], buf, 1000).map_err(|e| e.code())?;
        Ok(())
    }

    pub fn get_datetime(&mut self) -> Result<DateTime, esp_idf_svc::sys::esp_err_t> {
        let mut buf = [0u8; 7];
        self.read_regs(0x02, &mut buf)?;

        let seconds = bcd2_to_byte(buf[0] & 0x7F);
        let minutes = bcd2_to_byte(buf[1] & 0x7F);
        let hours = bcd2_to_byte(buf[2] & 0x3F);
        let day = bcd2_to_byte(buf[3] & 0x3F);
        let weekday = bcd2_to_byte(buf[4] & 0x07);
        let month = bcd2_to_byte(buf[5] & 0x1F);
        
        let century_bit = (buf[5] >> 7) & 1;
        let year_low = bcd2_to_byte(buf[6]);
        let year = if century_bit == 1 { 1900 + year_low as u16 } else { 2000 + year_low as u16 };

        Ok(DateTime { seconds, minutes, hours, day, weekday, month, year })
    }

    pub fn set_datetime(&mut self, dt: &DateTime) -> Result<(), esp_idf_svc::sys::esp_err_t> {
        let mut data = [0u8; 8];
        data[0] = 0x02;
        data[1] = byte_to_bcd2(dt.seconds);
        data[2] = byte_to_bcd2(dt.minutes);
        data[3] = byte_to_bcd2(dt.hours);
        data[4] = byte_to_bcd2(dt.day);
        data[5] = byte_to_bcd2(dt.weekday);
        
        let mut month_bcd = byte_to_bcd2(dt.month);
        if dt.year < 2000 { month_bcd |= 0x80; }
        data[6] = month_bcd;
        data[7] = byte_to_bcd2((dt.year % 100) as u8);

        self.i2c.write(self.addr, &data, 1000).map_err(|e| e.code())?;
        Ok(())
    }

    pub fn sync_system_time(&mut self) -> color_eyre::Result<()> {
        let dt = self.get_datetime().map_err(|e| color_eyre::eyre::eyre!("RTC read error: {}", e))?;
        
        // Convert to Unix timestamp
        let naive = chrono::NaiveDate::from_ymd_opt(dt.year as i32, dt.month as u32, dt.day as u32)
            .ok_or_eyre("Invalid date from RTC")?
            .and_hms_opt(dt.hours as u32, dt.minutes as u32, dt.seconds as u32)
            .ok_or_eyre("Invalid time from RTC")?;
        
        let timestamp = naive.and_utc().timestamp();
        
        unsafe {
            let tv = timeval {
                tv_sec: timestamp as _,
                tv_usec: 0,
            };
            let tz = timezone {
                tz_minuteswest: 0,
                tz_dsttime: 0,
            };
            if settimeofday(&tv, &tz) != 0 {
                return Err(color_eyre::eyre::eyre!("Failed to set system time"));
            }
        }
        
        Ok(())
    }
}

fn bcd2_to_byte(value: u8) -> u8 { ((value >> 4) * 10) + (value & 0x0F) }
fn byte_to_bcd2(value: u8) -> u8 { ((value / 10) << 4) | (value % 10) }
