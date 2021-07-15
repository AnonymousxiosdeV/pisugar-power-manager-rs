use crate::Result;
use chrono::prelude::*;
use chrono::{DateTime, Local, LocalResult, Utc};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display};

pub fn bcd_to_dec(bcd: u8) -> u8 {
    (bcd & 0x0F) + (((bcd & 0xF0) >> 4) * 10)
}

pub fn dec_to_bcd(dec: u8) -> u8 {
    dec % 10 + ((dec / 10) << 4)
}

/// RTC raw time, always UTC 24hr
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct RTCRawTime(pub [u8; 7]);

impl RTCRawTime {
    /// From raw sd3078 time
    pub fn from_raw(sd3078_raw: [u8; 7]) -> Self {
        Self(sd3078_raw)
    }

    /// From dec
    pub fn from_dec(dec: [u8; 7]) -> Self {
        let mut raw = [0; 7];
        for i in 0..7 {
            raw[i] = bcd_to_dec(dec[i]);
        }
        Self(raw)
    }

    /// Year, 2000-2099
    pub fn year(&self) -> u16 {
        bcd_to_dec(self.0[6]) as u16 + 2000
    }

    /// Month, 1-12
    pub fn month(&self) -> u8 {
        bcd_to_dec(self.0[5])
    }

    /// Day of month, 1-31
    pub fn day(&self) -> u8 {
        bcd_to_dec(self.0[4])
    }

    /// Weekday from sunday, 0-6
    pub fn weekday(&self) -> u8 {
        bcd_to_dec(self.0[3])
    }

    /// Hour, 0-23
    pub fn hour(&self) -> u8 {
        bcd_to_dec(self.0[2])
    }

    /// Minute, 0-59
    pub fn minute(&self) -> u8 {
        bcd_to_dec(self.0[1])
    }

    /// Second, 0-59
    pub fn second(&self) -> u8 {
        bcd_to_dec(self.0[0])
    }

    /// To dec
    pub fn to_dec(&self) -> [u8; 7] {
        [
            self.second(),
            self.minute(),
            self.hour(),
            self.weekday(),
            self.day(),
            self.month(),
            (self.year() - 2000) as u8,
        ]
    }
}

impl Display for RTCRawTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{},{},{},{},{},{},{}]",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6]
        )
    }
}

impl From<DateTime<Utc>> for RTCRawTime {
    fn from(dt: DateTime<Utc>) -> Self {
        let mut t = RTCRawTime([0; 7]);
        t.0[6] = dec_to_bcd((dt.year() % 100) as u8);
        t.0[5] = dec_to_bcd(dt.month() as u8);
        t.0[4] = dec_to_bcd(dt.day() as u8);
        t.0[3] = dec_to_bcd(dt.weekday().num_days_from_sunday() as u8);
        t.0[2] = dec_to_bcd(dt.hour() as u8);
        t.0[1] = dec_to_bcd(dt.minute() as u8);
        t.0[0] = dec_to_bcd(dt.second() as u8);
        t
    }
}

impl TryFrom<RTCRawTime> for DateTime<Utc> {
    type Error = ();

    fn try_from(t: RTCRawTime) -> std::result::Result<Self, Self::Error> {
        let sec = bcd_to_dec(t.0[0]) as u32;
        let min = bcd_to_dec(t.0[1]) as u32;
        let hour = bcd_to_dec(t.0[2]) as u32;
        let day_of_month = bcd_to_dec(t.0[4]) as u32;
        let month = bcd_to_dec(t.0[5]) as u32;
        let year = 2000 + bcd_to_dec(t.0[6]) as i32;

        let datetime = Utc.ymd_opt(year, month, day_of_month).and_hms_opt(hour, min, sec);
        match datetime {
            LocalResult::Single(datetime) => Ok(datetime),
            _ => Err(()),
        }
    }
}

impl From<DateTime<Local>> for RTCRawTime {
    fn from(dt: DateTime<Local>) -> Self {
        let dt: DateTime<Utc> = DateTime::from(dt);
        dt.into()
    }
}

impl TryFrom<RTCRawTime> for DateTime<Local> {
    type Error = ();

    fn try_from(t: RTCRawTime) -> std::result::Result<Self, Self::Error> {
        t.try_into().and_then(|dt: DateTime<Utc>| Ok(DateTime::from(dt)))
    }
}

/// RTC trait
pub trait RTC {
    /// Init
    fn init(&self, auto_power_on: bool, auto_wakeup_time: Option<DateTime<Local>>, wakeup_repeat: u8) -> Result<()>;

    /// Read RTC time
    fn read_time(&self) -> Result<RTCRawTime>;

    /// Write RTC time
    fn write_time(&self, time: RTCRawTime) -> Result<()>;

    /// Read alarm time
    fn read_alarm_time(&self) -> Result<RTCRawTime>;

    /// Write alarm time
    fn set_alarm(&self, time: RTCRawTime, weekday_repeat: u8) -> Result<()>;

    /// Is alarm enabled
    fn is_alarm_enable(&self) -> Result<bool>;

    /// Toggle alarm enabled
    fn toggle_alarm_enable(&self, enable: bool) -> Result<()>;

    fn read_alarm_flag(&self) -> Result<bool>;

    fn clear_alarm_flag(&self) -> Result<()>;

    fn toggle_frequency_alarm(&self, enable: bool) -> Result<()>;

    fn set_test_wake(&self) -> Result<()>;

    fn force_shutdown(&self) -> Result<()>;

    fn read_battery_low_flag(&self) -> Result<bool>;

    fn toggle_charging(&self, enable: bool) -> Result<()>;

    fn read_battery_high_flag(&self) -> Result<bool>;
}
