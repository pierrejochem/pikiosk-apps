//! One Open-Meteo `forecast` request, parsed into [`Forecast`].

use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::config::Config;
use crate::model::{Day, Forecast, Point, Stamp};

#[derive(Deserialize)]
struct Response {
    utc_offset_seconds: i64,
    current: Current,
    hourly: Hourly,
    daily: Daily,
}

#[derive(Deserialize)]
struct Current {
    temperature_2m: f32,
    relative_humidity_2m: f32,
    wind_speed_10m: f32,
    weather_code: u8,
}

/// Open-Meteo sends `null` for hours it has no value for.
#[derive(Deserialize)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<Option<f32>>,
    precipitation_probability: Vec<Option<f32>>,
    wind_speed_10m: Vec<Option<f32>>,
    wind_direction_10m: Vec<Option<f32>>,
    weather_code: Vec<Option<u8>>,
}

#[derive(Deserialize)]
struct Daily {
    time: Vec<String>,
    weather_code: Vec<Option<u8>>,
    temperature_2m_max: Vec<Option<f32>>,
    temperature_2m_min: Vec<Option<f32>>,
}

pub fn url(c: &Config) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={:.4}&longitude={:.4}\
         &current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code\
         &hourly=temperature_2m,precipitation_probability,wind_speed_10m,wind_direction_10m,weather_code\
         &daily=weather_code,temperature_2m_max,temperature_2m_min\
         &timezone=auto&forecast_days=8&wind_speed_unit=kmh",
        c.latitude, c.longitude
    )
}

pub fn fetch(c: &Config) -> Result<Forecast> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(20))).build().into();
    let body = agent.get(url(c)).call().context("request failed")?.body_mut().read_to_string().context("reading body")?;
    parse(&body)
}

pub fn parse(json: &str) -> Result<Forecast> {
    let r: Response = serde_json::from_str(json).context("unexpected response")?;
    let h = &r.hourly;
    let n = h.time.len();
    if n == 0 || [h.temperature_2m.len(), h.precipitation_probability.len(), h.wind_speed_10m.len(), h.wind_direction_10m.len(), h.weather_code.len()].iter().any(|&l| l != n) {
        bail!("hourly arrays differ in length");
    }
    let hourly = (0..n)
        .filter_map(|i| {
            Some(Point {
                at: Stamp::parse(&h.time[i])?,
                temp: h.temperature_2m[i].unwrap_or(0.0),
                precip: h.precipitation_probability[i].unwrap_or(0.0),
                wind: h.wind_speed_10m[i].unwrap_or(0.0),
                wind_dir: h.wind_direction_10m[i].unwrap_or(0.0),
                code: h.weather_code[i].unwrap_or(3),
            })
        })
        .collect();
    let d = &r.daily;
    let days = (0..d.time.len())
        .filter_map(|i| {
            Some(Day {
                date: Stamp::parse(&d.time[i])?,
                code: d.weather_code.get(i).copied().flatten().unwrap_or(3),
                max: d.temperature_2m_max.get(i).copied().flatten()?,
                min: d.temperature_2m_min.get(i).copied().flatten()?,
            })
        })
        .collect::<Vec<_>>();
    if days.is_empty() {
        bail!("no daily data");
    }
    Ok(Forecast {
        utc_offset: r.utc_offset_seconds,
        temp: r.current.temperature_2m,
        humidity: r.current.relative_humidity_2m,
        wind: r.current.wind_speed_10m,
        code: r.current.weather_code,
        hourly,
        days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "utc_offset_seconds": 3600,
        "current": {"time": "2026-10-25T18:00", "temperature_2m": 9.2, "relative_humidity_2m": 83, "wind_speed_10m": 23.4, "weather_code": 61},
        "hourly": {
            "time": ["2026-10-25T18:00", "2026-10-25T19:00"],
            "temperature_2m": [9.2, null],
            "precipitation_probability": [83, 52],
            "wind_speed_10m": [23.4, 23.0],
            "wind_direction_10m": [225, 230],
            "weather_code": [61, null]
        },
        "daily": {
            "time": ["2026-10-25", "2026-10-26"],
            "weather_code": [61, 3],
            "temperature_2m_max": [12.1, 12.0],
            "temperature_2m_min": [7.9, null]
        }
    }"#;

    #[test]
    fn parses_a_response_and_tolerates_nulls() {
        let f = parse(SAMPLE).unwrap();
        assert_eq!((f.temp, f.humidity, f.code, f.utc_offset), (9.2, 83.0, 61, 3600));
        assert_eq!(f.hourly.len(), 2);
        assert_eq!(f.hourly[1].temp, 0.0, "null temperature becomes 0");
        assert_eq!(f.hourly[1].code, 3, "null code becomes overcast");
        assert_eq!(f.hourly[0].precip, 83.0);
        assert_eq!(f.days.len(), 1, "a day without a low is dropped");
        assert_eq!(f.days[0].max, 12.1);
    }

    #[test]
    fn rejects_garbage_and_ragged_arrays() {
        assert!(parse("{}").is_err());
        assert!(parse(&SAMPLE.replace("[83, 52]", "[83]")).is_err());
    }

    #[test]
    fn url_carries_the_location() {
        let u = url(&Config { name: "x".into(), latitude: 53.65, longitude: 10.15 });
        assert!(u.contains("latitude=53.6500&longitude=10.1500"));
        assert!(u.contains("timezone=auto"));
    }
}
