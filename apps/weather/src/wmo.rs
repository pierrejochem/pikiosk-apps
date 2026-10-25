//! WMO weather codes (as Open-Meteo reports them) in German, plus the icon to draw.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconKind {
    Sun,
    PartlyCloudy,
    Cloud,
    Rain,
    Snow,
    Thunder,
    Fog,
}

pub fn describe(code: u8) -> (&'static str, IconKind) {
    use IconKind::*;
    match code {
        0 => ("Klar", Sun),
        1 => ("Überwiegend klar", PartlyCloudy),
        2 => ("Teils bewölkt", PartlyCloudy),
        3 => ("Bewölkt", Cloud),
        45 | 48 => ("Nebel", Fog),
        51 => ("Leichter Nieselregen", Rain),
        53 => ("Mäßiger Nieselregen", Rain),
        55 => ("Starker Nieselregen", Rain),
        56 | 57 => ("Gefrierender Nieselregen", Rain),
        61 => ("Leichter Regen", Rain),
        63 => ("Mäßiger Regen", Rain),
        65 => ("Starker Regen", Rain),
        66 | 67 => ("Gefrierender Regen", Rain),
        71 => ("Leichter Schneefall", Snow),
        73 => ("Mäßiger Schneefall", Snow),
        75 => ("Starker Schneefall", Snow),
        77 => ("Schneegriesel", Snow),
        80 => ("Leichte Regenschauer", Rain),
        81 => ("Mäßige Regenschauer", Rain),
        82 => ("Heftige Regenschauer", Rain),
        85 | 86 => ("Schneeschauer", Snow),
        95 => ("Gewitter", Thunder),
        96 | 99 => ("Gewitter mit Hagel", Thunder),
        _ => ("Unbekannt", Cloud),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_codes() {
        assert_eq!(describe(0), ("Klar", IconKind::Sun));
        assert_eq!(describe(63), ("Mäßiger Regen", IconKind::Rain));
        assert_eq!(describe(95).1, IconKind::Thunder);
    }

    #[test]
    fn unknown_code_does_not_panic() {
        assert_eq!(describe(200).1, IconKind::Cloud);
    }
}
