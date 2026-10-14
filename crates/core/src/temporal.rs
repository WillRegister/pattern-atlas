use crate::model::{LeapDay, Settings};
use anyhow::{ensure, Result};
use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Serialize;

pub fn validate_date(date: NaiveDate) -> Result<()> {
    ensure!(
        (1..=9998).contains(&date.year()),
        "Supported calendar years: 1 through 9998"
    );
    Ok(())
}
pub fn parse_date(text: &str) -> Result<NaiveDate> {
    ensure!(text.len() == 10, "Use YYYY-MM-DD");
    let d = NaiveDate::parse_from_str(text, "%Y-%m-%d")?;
    validate_date(d)?;
    Ok(d)
}
pub fn utc_instant(date: NaiveDate, time: NaiveTime, timezone: &str) -> Result<DateTime<Utc>> {
    validate_date(date)?;
    let tz: chrono_tz::Tz = timezone.parse()?;
    match tz.from_local_datetime(&date.and_time(time)) {
        LocalResult::Single(d) => Ok(d.with_timezone(&Utc)),
        LocalResult::Ambiguous(_, _) => {
            anyhow::bail!("Ambiguous local time during DST transition; use an explicit UTC time")
        }
        LocalResult::None => anyhow::bail!("This local time does not exist during DST transition"),
    }
}
fn anniversary(date: NaiveDate, year: i32, policy: LeapDay) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, date.month(), date.day()).unwrap_or_else(|| match policy {
        LeapDay::February28 => NaiveDate::from_ymd_opt(year, 2, 28).unwrap(),
        LeapDay::March1 => NaiveDate::from_ymd_opt(year, 3, 1).unwrap(),
    })
}
pub fn next_anniversary(date: NaiveDate, active: NaiveDate, policy: LeapDay) -> NaiveDate {
    // No recurrence before the original event. Today counts as the next occurrence.
    let year = active.year().max(date.year());
    let candidate = anniversary(date, year, policy);
    if candidate >= active {
        candidate
    } else {
        anniversary(date, year + 1, policy)
    }
}
pub fn previous_anniversary(
    date: NaiveDate,
    active: NaiveDate,
    policy: LeapDay,
) -> Option<NaiveDate> {
    if active < date {
        return None;
    }
    let candidate = anniversary(date, active.year(), policy);
    let previous = if candidate <= active {
        candidate
    } else {
        anniversary(date, active.year() - 1, policy)
    };
    (previous >= date).then_some(previous)
}

#[derive(Debug, Clone, Serialize)]
pub struct DateRelation {
    pub original: Span,
    pub previous: Option<NaiveDate>,
    pub next: NaiveDate,
    pub since: Option<Span>,
    pub until: Span,
}
/// Original intervals follow end inclusion. Recurrence countdowns always use civil-day offsets.
pub fn relation(event: NaiveDate, reference: NaiveDate, settings: &Settings) -> DateRelation {
    let previous = previous_anniversary(event, reference, settings.leap_day);
    let next = next_anniversary(event, reference, settings.leap_day);
    let mut calendar = settings.clone();
    calendar.include_end = false;
    DateRelation {
        original: span(event, reference, settings),
        previous,
        next,
        since: previous.map(|date| span(date, reference, &calendar)),
        until: span(reference, next, &calendar),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnniversaryPair {
    pub previous: Option<Span>,
    pub next: Span,
}
pub fn anniversary_pair(
    a: NaiveDate,
    b: NaiveDate,
    reference: NaiveDate,
    settings: &Settings,
) -> AnniversaryPair {
    let a = relation(a, reference, settings);
    let b = relation(b, reference, settings);
    let mut civil = settings.clone();
    civil.include_end = false;
    AnniversaryPair {
        previous: a.previous.zip(b.previous).map(|(a, b)| span(a, b, &civil)),
        next: span(a.next, b.next, &civil),
    }
}
pub fn add_months(date: NaiveDate, months: i32) -> Result<NaiveDate> {
    let index = date.year() * 12 + date.month0() as i32 + months;
    let year = index.div_euclid(12);
    let month = index.rem_euclid(12) as u32 + 1;
    ensure!((1..=9999).contains(&year), "Calendar shift is out of range");
    let mut day = date.day();
    loop {
        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
            return Ok(d);
        }
        day -= 1;
    }
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Span {
    pub sign: i64,
    pub years: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub total_days: i64,
}
impl Span {
    pub fn label(&self, s: &Settings) -> String {
        let mut parts = vec![];
        for (on, value, unit) in [
            (s.years, self.years, "y"),
            (s.months, self.months, "mo"),
            (s.weeks, self.weeks, "w"),
            (s.days, self.days, "d"),
        ] {
            if on {
                parts.push(format!("{value}{unit}"));
            }
        }
        if !s.days && self.days > 0 {
            parts.push(format!("(+{}d remainder)", self.days));
        }
        format!(
            "{}{}",
            if self.sign < 0 { "−" } else { "" },
            parts.join(" ")
        )
    }
}
pub fn span(a: NaiveDate, b: NaiveDate, s: &Settings) -> Span {
    let sign = if b < a { -1 } else { 1 };
    let (start, mut end) = if a <= b { (a, b) } else { (b, a) };
    if s.include_end {
        end += Duration::days(1);
    }
    let total_days = (end - start).num_days();
    let mut full_months =
        (end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32;
    if add_months(start, full_months).unwrap() > end {
        full_months -= 1;
    }
    let years = if s.years { full_months / 12 } else { 0 };
    let months = if s.months {
        full_months - years * 12
    } else {
        0
    };
    let cursor = add_months(start, years * 12 + months).unwrap();
    let remaining = (end - cursor).num_days();
    let weeks = if s.weeks { remaining / 7 } else { 0 };
    Span {
        sign,
        years: years.into(),
        months: months.into(),
        weeks,
        days: remaining - weeks * 7,
        total_days,
    }
}
