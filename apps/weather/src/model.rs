//! The forecast as the UI wants it: plain structs, local wall-clock times, no
//! time crate. Open-Meteo is asked for `timezone=auto`, so every timestamp in
//! the response is already local time and `utc_offset_seconds` turns the
//! system clock into the same zone.

use crate::wmo::{self, IconKind};

/// A local date and hour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stamp {
    pub y: i32,
    pub m: u32,
    pub d: u32,
    pub h: u32,
}

impl Stamp {
    /// Parses `2026-10-25T18:00` (or a bare `2026-10-25`).
    pub fn parse(s: &str) -> Option<Stamp> {
        let (date, time) = s.split_once('T').unwrap_or((s, "00:00"));
        let mut d = date.split('-');
        let (y, m, day) = (d.next()?.parse().ok()?, d.next()?.parse().ok()?, d.next()?.parse().ok()?);
        let h = time.split(':').next()?.parse().ok()?;
        Some(Stamp { y, m, d: day, h })
    }

    /// 0 = Monday ... 6 = Sunday.
    pub fn weekday(&self) -> usize {
        // 1970-01-01 was a Thursday (3).
        (days_from_civil(self.y, self.m, self.d) + 3).rem_euclid(7) as usize
    }

    pub fn same_day(&self, o: &Stamp) -> bool {
        (self.y, self.m, self.d) == (o.y, o.m, o.d)
    }
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((y + i64::from(m <= 2)) as i32, m, d)
}

/// Local time from Unix seconds and a UTC offset: the stamp plus the minute.
pub fn local_from_unix(unix: i64, offset: i64) -> (Stamp, u32) {
    let t = unix + offset;
    let (y, m, d) = civil_from_days(t.div_euclid(86_400));
    let secs = t.rem_euclid(86_400);
    (Stamp { y, m, d, h: (secs / 3600) as u32 }, ((secs % 3600) / 60) as u32)
}

pub const WEEKDAYS_LONG: [&str; 7] = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"];
pub const WEEKDAYS_SHORT: [&str; 7] = ["Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa.", "So."];

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub at: Stamp,
    pub temp: f32,
    pub precip: f32,
    pub wind: f32,
    /// Where the wind blows from, in degrees.
    pub wind_dir: f32,
    pub code: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Day {
    pub date: Stamp,
    pub code: u8,
    pub max: f32,
    pub min: f32,
}

impl Day {
    pub fn icon(&self) -> IconKind {
        wmo::describe(self.code).1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Forecast {
    pub utc_offset: i64,
    pub temp: f32,
    pub humidity: f32,
    pub wind: f32,
    pub code: u8,
    pub hourly: Vec<Point>,
    pub days: Vec<Day>,
}

pub const CHART_POINTS: usize = 8;

impl Forecast {
    /// Index of the hourly entry for `now`, or 0 when the data does not reach it.
    pub fn hour_index(&self, now: Stamp) -> usize {
        self.hourly.iter().position(|p| p.at == now).unwrap_or(0)
    }

    /// The precipitation chance for the current hour.
    pub fn precip_now(&self, now: Stamp) -> f32 {
        self.hourly.get(self.hour_index(now)).map_or(0.0, |p| p.precip)
    }

    /// Eight points three hours apart. Today starts at the next hour after
    /// `now`; any other day starts at midnight.
    pub fn chart_points(&self, day: usize, now: Stamp) -> Vec<Point> {
        let start = if day == 0 {
            self.hour_index(now) + 1
        } else {
            let Some(d) = self.days.get(day) else { return Vec::new() };
            match self.hourly.iter().position(|p| p.at.same_day(&d.date)) {
                Some(i) => i,
                None => return Vec::new(),
            }
        };
        (0..CHART_POINTS).filter_map(|i| self.hourly.get(start + 3 * i).cloned()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(y: i32, m: u32, d: u32, h: u32) -> Stamp {
        Stamp { y, m, d, h }
    }

    #[test]
    fn parses_stamps() {
        assert_eq!(Stamp::parse("2026-10-25T18:00"), Some(st(2026, 10, 25, 18)));
        assert_eq!(Stamp::parse("2026-10-25"), Some(st(2026, 10, 25, 0)));
        assert_eq!(Stamp::parse("nonsense"), None);
    }

    #[test]
    fn weekday_matches_the_screenshots() {
        // 2026-10-25 is a Sunday.
        assert_eq!(st(2026, 10, 25, 0).weekday(), 6);
        assert_eq!(st(2026, 10, 26, 0).weekday(), 0);
        assert_eq!(st(1970, 1, 1, 0).weekday(), 3);
    }

    #[test]
    fn unix_to_local_applies_the_offset_and_rolls_the_day() {
        // 2026-10-25 17:00 UTC is 18:00 in Hamburg (CET, +1 h after DST ends).
        let unix = days_from_civil(2026, 10, 25) * 86_400 + 17 * 3600 + 30 * 60;
        assert_eq!(local_from_unix(unix, 3600), (st(2026, 10, 25, 18), 30));
        assert_eq!(local_from_unix(unix + 6 * 3600, 3600).0, st(2026, 10, 26, 0));
    }

    fn forecast() -> Forecast {
        // 3 days of hourly data starting 2026-10-25 00:00.
        let mut hourly = Vec::new();
        for day in 0..3u32 {
            for h in 0..24 {
                hourly.push(Point { at: st(2026, 10, 25 + day, h), temp: (day * 24 + h) as f32, precip: 0.0, wind: 0.0, wind_dir: 0.0, code: 0 });
            }
        }
        let days = (0..3).map(|d| Day { date: st(2026, 10, 25 + d, 0), code: 0, max: 0.0, min: 0.0 }).collect();
        Forecast { utc_offset: 3600, temp: 9.0, humidity: 83.0, wind: 23.0, code: 61, hourly, days }
    }

    #[test]
    fn today_starts_after_now_and_steps_three_hours() {
        let pts = forecast().chart_points(0, st(2026, 10, 25, 18));
        let hours: Vec<u32> = pts.iter().map(|p| p.at.h).collect();
        assert_eq!(hours, [19, 22, 1, 4, 7, 10, 13, 16]);
        assert_eq!(pts[2].at.d, 26);
    }

    #[test]
    fn later_days_start_at_midnight() {
        let pts = forecast().chart_points(1, st(2026, 10, 25, 18));
        let hours: Vec<u32> = pts.iter().map(|p| p.at.h).collect();
        assert_eq!(hours, [0, 3, 6, 9, 12, 15, 18, 21]);
        assert!(pts.iter().all(|p| p.at.d == 26));
    }

    #[test]
    fn data_that_runs_out_gives_fewer_points() {
        assert_eq!(forecast().chart_points(0, st(2026, 10, 27, 20)).len(), 1);
        assert!(forecast().chart_points(9, st(2026, 10, 25, 18)).is_empty());
    }
}
