use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;

use crate::{Error, Value};

pub fn now_ns() -> Result<Value, Error> {
    Ok(Utc::now().timestamp_nanos().into())
}

pub fn date(value: Value) -> Result<Value, Error> {
    match value {
        Value::Number(n) if n.is_i64() => {
            let datetime = Utc.timestamp_nanos(n.try_into_i64()?);
            Ok(vec![
                datetime.year(),
                datetime.month() as i32,
                datetime.day() as i32,
            ]
            .into())
        }
        Value::Array(v) => match &v[..] {
            [nanos, tz] => {
                let nanos = nanos
                    .as_i64()
                    .ok_or_else(|| Error::InvalidType("i64", nanos.clone()))?;
                let v = match tz
                    .as_str()
                    .ok_or_else(|| Error::InvalidType("string", tz.clone()))?
                {
                    "UTC" | "" => {
                        let datetime = Utc.timestamp_nanos(nanos);
                        vec![
                            datetime.year(),
                            datetime.month() as i32,
                            datetime.day() as i32,
                        ]
                    }
                    "Local" => {
                        let datetime = Local.timestamp_nanos(nanos);
                        vec![
                            datetime.year(),
                            datetime.month() as i32,
                            datetime.day() as i32,
                        ]
                    }
                    iana => {
                        let datetime = iana
                            .parse::<Tz>()
                            .map_err(Error::UnknownTimezone)?
                            .timestamp_nanos(nanos);
                        vec![
                            datetime.year(),
                            datetime.month() as i32,
                            datetime.day() as i32,
                        ]
                    }
                };
                Ok(v.into())
            }
            v => Err(Error::InvalidType("i64 or array[ns, tz]", v.into())),
        },
        v => Err(Error::InvalidType("i64 or array[ns, tz]", v)),
    }
}

pub fn clock(value: Value) -> Result<Value, Error> {
    match value {
        Value::Number(n) if n.is_i64() => {
            let datetime = Utc.timestamp_nanos(n.try_into_i64()?);
            Ok(vec![datetime.hour(), datetime.minute(), datetime.second()].into())
        }
        Value::Array(v) => match &v[..] {
            [nanos, tz] => {
                let nanos = nanos
                    .as_i64()
                    .ok_or_else(|| Error::InvalidType("i64", nanos.clone()))?;
                let v = match tz
                    .as_str()
                    .ok_or_else(|| Error::InvalidType("string", tz.clone()))?
                {
                    "UTC" | "" => {
                        let datetime = Utc.timestamp_nanos(nanos);
                        vec![datetime.hour(), datetime.minute(), datetime.second()]
                    }
                    "Local" => {
                        let datetime = Local.timestamp_nanos(nanos);
                        vec![datetime.hour(), datetime.minute(), datetime.second()]
                    }
                    iana => {
                        let datetime = iana
                            .parse::<Tz>()
                            .map_err(Error::UnknownTimezone)?
                            .timestamp_nanos(nanos);
                        vec![datetime.hour(), datetime.minute(), datetime.second()]
                    }
                };
                Ok(v.into())
            }
            v => Err(Error::InvalidType("i64 or array[ns, tz]", v.into())),
        },
        v => Err(Error::InvalidType("i64 or array[ns, tz]", v)),
    }
}

pub fn weekday(value: Value) -> Result<Value, Error> {
    match value {
        Value::Number(n) if n.is_i64() => {
            let datetime = Utc.timestamp_nanos(n.try_into_i64()?);
            Ok(vec![datetime.hour(), datetime.minute(), datetime.second()].into())
        }
        Value::Array(v) => match &v[..] {
            [nanos, tz] => {
                let nanos = nanos
                    .as_i64()
                    .ok_or_else(|| Error::InvalidType("i64", nanos.clone()))?;
                let v = match tz
                    .as_str()
                    .ok_or_else(|| Error::InvalidType("string", tz.clone()))?
                {
                    "UTC" | "" => {
                        let datetime = Utc.timestamp_nanos(nanos);
                        weekday_to_string(datetime.weekday())
                    }
                    "Local" => {
                        let datetime = Local.timestamp_nanos(nanos);
                        weekday_to_string(datetime.weekday())
                    }
                    iana => {
                        let datetime = iana
                            .parse::<Tz>()
                            .map_err(Error::UnknownTimezone)?
                            .timestamp_nanos(nanos);
                        weekday_to_string(datetime.weekday())
                    }
                };
                Ok(v.into())
            }
            v => Err(Error::InvalidType("i64 or array[ns, tz]", v.into())),
        },
        v => Err(Error::InvalidType("i64 or array[ns, tz]", v)),
    }
}

fn weekday_to_string(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "Monday".to_string(),
        Weekday::Tue => "Tuesday".to_string(),
        Weekday::Wed => "Wednesday".to_string(),
        Weekday::Thu => "Thursday".to_string(),
        Weekday::Fri => "Friday".to_string(),
        Weekday::Sat => "Saturday".to_string(),
        Weekday::Sun => "Sunday".to_string(),
    }
}

pub fn parse_rfc3339_ns(value: Value) -> Result<Value, Error> {
    let string = value.try_into_string()?;
    let datetime = DateTime::parse_from_rfc3339(&string).map_err(Error::ParseDatetime)?;
    Ok(datetime.timestamp_nanos().into())
}
