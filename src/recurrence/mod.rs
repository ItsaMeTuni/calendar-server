//! This module does everything related to event recurrences, from RRULE parsing
//! to calculating recurring event instances.

use chrono::{Date, Utc, NaiveDate, Duration, Datelike, Weekday, Month, IsoWeek, ParseError};

use self::helpers::NaiveDateHelpers;
use std::fmt::{Formatter, Display};
use crate::event::EventRecurring;

mod recurrence_parser;
mod helpers;
pub mod serde;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum RecurrenceFreq
{
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum RecurrenceLimit
{
    Indefinite,
    Date(NaiveDate),
    Count(u32),
}


/// An event's recurrence rule, this is used by `Event.generate_instances`
/// to figure out when event instances will happen.
/// This is basically a data structure to represent an
/// RRULE as defined in RFC 5545.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RecurrenceRule
{
    frequency: RecurrenceFreq,
    interval: i32,
    limit: RecurrenceLimit,

    by_month: Option<Vec<Month>>,
    by_week_no: Option<Vec<i32>>,
    by_year_day: Option<Vec<i32>>,
    by_month_day: Option<Vec<i32>>,
    by_day: Option<Vec<Weekday>>,
    by_set_pos: Option<Vec<i32>>,
}

impl RecurrenceRule
{
    /// Parses an RRULE string.
    pub fn new(rrule: &str) -> Result<Self, recurrence_parser::RRuleParseError>
    {
        let rule = recurrence_parser::parse(rrule)?;
        Ok(rule)
    }

    fn set_interval(mut self, interval: i32) -> Self
    {
        self.interval = interval;

        self
    }

    fn set_limit_date(mut self, date: NaiveDate) -> Self
    {
        self.limit = RecurrenceLimit::Date(date);

        self
    }

    fn set_limit_count(mut self, count: u32) -> Self
    {
        self.limit = RecurrenceLimit::Count(count);

        self
    }

    /// Returns a clone of this recurrence rule with
    /// inferred values if they're not already set.
    ///
    /// E.g.: if not already specified, BYDAY is inferred
    /// to be the same weekday as `starting_at` when
    /// FREQ=WEEKLY.
    fn infer_stuff(&self, start_date: NaiveDate) -> RecurrenceRule
    {
        let mut new_rule = self.clone();

        // Infer BYDAY if recurrence is weekly
        if new_rule.frequency == RecurrenceFreq::Weekly && new_rule.by_day.is_none()
        {
            new_rule.by_day = Some(vec![start_date.weekday()]);
        }

        // Infer BYMONTHDAY if recurrence is monthly
        if new_rule.frequency == RecurrenceFreq::Monthly && new_rule.by_month_day.is_none() && new_rule.by_day.is_none()
        {
            new_rule.by_month_day = Some(vec![start_date.day() as i32]);
        }

        if new_rule.frequency == RecurrenceFreq::Yearly
        {
            // Infer BYMONTHDAY if BYMONTH is set
            if new_rule.by_month.is_some()
            {
                if new_rule.by_month_day.is_none()
                {
                    new_rule.by_month_day = Some(vec![start_date.day() as i32]);
                }
            }
            // Infer BYDAY if BYWEEKNO is set
            else if new_rule.by_week_no.is_some()
            {
                if new_rule.by_day.is_none()
                {
                    new_rule.by_day = Some(vec![start_date.weekday()]);
                }
            }
            // Infer BYYEARDAY if it's not set
            else if new_rule.by_year_day.is_none()
            {
                new_rule.by_year_day = Some(vec![start_date.year_day() as i32]);
            }
        }

        new_rule
    }

    /// Calculate event instances based on this rule.
    ///
    ///
    /// Some rule properties might be inferred from `starting_at` if
    /// they're not present in the rule (e.g. if not already specified,
    /// BYDAY is inferred to be the same weekday as `starting_at`
    /// when FREQ=WEEKLY). You don't really have to worry about this
    /// unless you suspect there might be a bug with the inference
    /// algorithm. If you do, look at `infer_stuff`.
    pub fn calculate_instances(&self, from: NaiveDate, to: NaiveDate, starting_at: NaiveDate) -> RRuleInstances
    {
        RRuleInstances::new(self.infer_stuff(starting_at), from, to, starting_at)
    }

    fn check_by_month(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_month) = &self.by_month
        {
            by_month
                .iter()
                .find(|x| x.number_from_month() == date.month())
                .is_none()
        }
        else
        {
            true
        }
    }


    /// Check if `date` fits into the BYWEEKNO property of
    /// this rule.
    fn check_by_week_no(&self, _date: &NaiveDate) -> bool
    {
        if let Some(_by_week_no) = &self.by_week_no
        {
            if self.frequency != RecurrenceFreq::Yearly
            {
                panic!("by_week_no can only be used in a YEARLY recurrence.");
            }

            // TODO: implement this
            unimplemented!()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYYEARDAY property of
    /// this rule.
    fn check_by_year_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_year_day) = &self.by_year_day
        {
            if matches!(self.frequency, RecurrenceFreq::Daily | RecurrenceFreq::Weekly | RecurrenceFreq::Monthly)
            {
                panic!("by_year_day cannot be used in DAILY, WEEKLY, and MONTHLY recurrences.");
            }

            let year_day = date.year_day() as i32;
            by_year_day.iter().find(|x| **x == year_day).is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYMONTHDAY property of
    /// this rule.
    fn check_by_month_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_month_day) = &self.by_month_day
        {
            if matches!(self.frequency, RecurrenceFreq::Weekly)
            {
                panic!("by_month_day cannot be used in WEEKLY recurrences.");
            }

            let month_day = date.day() as i32;
            by_month_day.iter().find(|x| **x == month_day).is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYDAY property of
    /// this rule.
    fn check_by_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_day) = &self.by_day
        {
            by_day
                .iter()
                .find(|x| **x == date.weekday())
                .is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYSETPOS property of
    /// this rule.
    fn check_by_set_pos(&self, _date: &NaiveDate) -> bool
    {
        if let Some(_by_set_pos) = &self.by_set_pos
        {
            // TODO: implement this
            unimplemented!()
        }
        else
        {
            true
        }
    }

}

/// Calculates the recurrence instances for an event. I.e finds out the dates in which a recurring event
/// happens.
///
/// `starting_at` is the start date of the event. The date of the "original" event.
/// The function will only return dates between `from` and `to` (both inclusive).
///
///
/// ## How it works
///
/// Basically, we iterate through each date from `starting_at` until `to` and check if the
/// date matches the recurrence rule. If the date matches the rule and is between `from`
/// and `to` (both inclusive), we add it to the results vector.
///
/// ## A note on performance
/// This event is not very performant, it has an O(n) complexity where n is the number of days between
/// `starting_at` and `to`, so if `starting_at` is 2020-01-01 and `to` is 2021-01-01 the loop will execute 356
/// times. This doesn't seem so bad but if you have this function being called many times a second for events
/// a few years in the past this can quickly become a bottleneck. It works this way because I don't know any other
/// way to calculate the recurrence dates while taking into account all parameters as defined in RFC 5545. There
/// might be a better way to do this, but I don't know about it.
pub struct RRuleInstances
{
    rule: RecurrenceRule,
    from: NaiveDate,
    to: NaiveDate,
    starting_at: NaiveDate,
    instance_count: u32,
    last_instance_date: NaiveDate,
    current_date: NaiveDate,
}

impl RRuleInstances
{
    pub fn new(rule: RecurrenceRule, from: NaiveDate, to: NaiveDate, starting_at: NaiveDate) -> RRuleInstances
    {
        RRuleInstances {
            rule,
            from,
            to,
            starting_at,
            instance_count: 0,
            last_instance_date: starting_at,
            current_date: starting_at,
        }
    }
}

impl Iterator for RRuleInstances
{
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            let mut is_match = false;

            // Order matters here! This should be in the same order
            // as specified in RFC 5545
            let fits_into_rule =
                self.rule.check_by_month(&self.current_date)
                    && self.rule.check_by_week_no(&self.current_date)
                    && self.rule.check_by_year_day(&self.current_date)
                    && self.rule.check_by_month_day(&self.current_date)
                    && self.rule.check_by_day(&self.current_date)
                    && self.rule.check_by_set_pos(&self.current_date);

            match self.rule.limit
            {
                RecurrenceLimit::Indefinite => {},
                RecurrenceLimit::Date(date) =>
                    if self.current_date > date
                    {
                        break;
                    },
                RecurrenceLimit::Count(count) =>
                    if self.instance_count >= count
                    {
                        break;
                    },
            };

            if self.current_date > self.to
            {
                break;
            }
            else if fits_into_rule
            {
                let freq_diff = match self.rule.frequency
                {
                    RecurrenceFreq::Daily => (self.current_date - self.last_instance_date).num_days(),
                    RecurrenceFreq::Weekly => calc_uniq_weeks_between(self.current_date, self.last_instance_date),
                    RecurrenceFreq::Monthly => {
                        if self.last_instance_date.month() > self.current_date.month()
                        {
                            (self.current_date.month() + 12 - self.last_instance_date.month()) as i64
                        }
                        else
                        {
                            (self.current_date.month() - self.last_instance_date.month()) as i64
                        }
                    },
                    RecurrenceFreq::Yearly => (self.current_date.year() - self.last_instance_date.year()) as i64,
                };

                if freq_diff >= self.rule.interval as i64 || freq_diff == 0
                {
                    self.instance_count += 1;

                    self.last_instance_date = self.current_date;

                    is_match = self.current_date >= self.from;
                }
            }

            self.current_date += Duration::days(self.rule.interval as i64);

            if is_match
            {
                return Some(self.last_instance_date);
            }
        }

        None
    }
}

/// Calculates how many different weeks there are between
/// a and b. Positive if a > b, negative if a < b.
///
/// **IMPORTANT:** this does not calculate a week as exactly 7
/// days! If `a` is 2020-01-21 (Tue) and `b` is 2020-01-01 (Wed),
/// this function will return 4.
fn calc_uniq_weeks_between(a: NaiveDate, b: NaiveDate) -> i64
{
    let mut count = 0;

    let days_until_monday = a.iter_days().take_while(|x| x.weekday() != Weekday::Mon).count();
    if days_until_monday > 0
    {
        count += 1;
    }

    let monday_date = a.iter_days().skip(days_until_monday).next().unwrap();

    (monday_date - b).num_weeks()
}

impl Display for RecurrenceRule
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        let freq = format!("FREQ={}", self.frequency);

        let interval = if self.interval > 1
        {
            Some(format!("INTERVAL={}", self.interval))
        }
        else
        {
            None
        };

        let by_year_day = self.by_year_day.clone()
            .map(|x| format!("BYYEARDAY={}", vec_to_str(x)));

        let by_week_no = self.by_week_no.clone()
            .map(|x| format!("BYWEEKNO={}", vec_to_str(x)));

        let by_month_day = self.by_month_day.clone()
            .map(|x| format!("BYMONTHDAY={}", vec_to_str(x)));

        let by_set_pos = self.by_set_pos.clone()
            .map(|x| format!("BYSETPOS={}", vec_to_str(x)));

        let by_month = self.by_month.clone()
            .map(|x| x.iter()
                .map(|x| x.number_from_month().to_string())
                .collect::<Vec<String>>()
                .join(",")
            )
            .map(|x| format!("BYMONTH={}", x));

        let by_day = self.by_day.clone()
            .map(|x| x.iter()
                .map(|x| match x
                {
                    Weekday::Mon => "MO",
                    Weekday::Tue => "TU",
                    Weekday::Wed => "WE",
                    Weekday::Thu => "TH",
                    Weekday::Fri => "FR",
                    Weekday::Sat => "SA",
                    Weekday::Sun => "SU",
                })
                .collect::<Vec<&str>>()
                .join(",")
            )
            .map(|x| format!("BYDAY={}", x));


        let limit = match self.limit
        {
            RecurrenceLimit::Indefinite => None,
            RecurrenceLimit::Date(date) => Some(format!("UNTIL={}", date.format("%Y%m%d"))),
            RecurrenceLimit::Count(count) => Some(format!("COUNT={}", count)),
        };

        let string = vec![Some(freq), interval, by_year_day, by_day, by_week_no, by_month_day, by_set_pos, by_month, limit]
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<String>>()
            .join(";");

        f.write_str(&string)
    }
}

fn vec_to_str<T: Display>(vec: Vec<T>) -> String
{
    vec.iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(",")
}

impl Display for RecurrenceFreq
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        let string = match self
        {
            RecurrenceFreq::Daily => "DAILY",
            RecurrenceFreq::Weekly => "WEEKLY",
            RecurrenceFreq::Monthly => "MONTHLY",
            RecurrenceFreq::Yearly => "YEARLY",
        };

        f.write_str(string)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use itertools::Itertools;

    #[test]
    fn calc_recurrences_weekly_indefinite()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Indefinite,
            by_day: Some(vec![start_date.weekday()]),
            ..RecurrenceRule::default()
        };

        let result = rule.calculate_instances(
            start_date,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        ).collect_vec();

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 22),
            NaiveDate::from_ymd(2020, 1, 29),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_weekly_w_date_limit()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Date(NaiveDate::from_ymd(2020, 1, 15)),
            by_day: Some(vec![start_date.weekday()]),
            ..RecurrenceRule::default()
        };

        let result = rule.calculate_instances(
            start_date,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        ).collect_vec();

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_weekly_w_count_limit()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Count(4),
            by_day: Some(vec![start_date.weekday()]),
            ..RecurrenceRule::default()
        };

        let result = rule.calculate_instances(
            start_date,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        ).collect_vec();

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 22),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_every_two_weeks()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            interval: 2,
            by_day: Some(vec![start_date.weekday()]),
            ..RecurrenceRule::default()
        };

        let result = rule.calculate_instances(
            start_date,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        ).collect_vec();

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 29),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn infer_by_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=WEEKLY").unwrap().infer_stuff(start_date);

        assert_eq!(rule.by_day, Some(vec![Weekday::Sat]));
    }

    #[test]
    fn infer_by_month_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=MONTHLY").unwrap().infer_stuff(start_date);

        assert_eq!(rule.by_month_day, Some(vec![26]));
    }

    #[test]
    fn yearly_infer_by_month_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY;BYMONTH=2").unwrap().infer_stuff(start_date);

        assert_eq!(rule.by_month_day, Some(vec![26]));
    }

    #[test]
    fn yearly_infer_by_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY;BYWEEKNO=2,4,6").unwrap().infer_stuff(start_date);

        assert_eq!(rule.by_day, Some(vec![Weekday::Sat]));
    }

    #[test]
    fn yearly_infer_by_year_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY").unwrap().infer_stuff(start_date);

        assert_eq!(rule.by_year_day, Some(vec![start_date.year_day() as i32]));
    }
}
