use chrono::{NaiveDate, Weekday, ParseResult, Month};
use super::{RecurrenceLimit, RecurrenceFreq, RecurrenceRule};

use std::collections::HashMap;

use num_traits::cast::FromPrimitive;

#[derive(Error, Debug)]
pub enum RRuleParseError
{
    #[error("Property {0} is duplicated.")]
    DuplicateProperty(&'static str),

    #[error("Property {0} has an invalid value.")]
    InvalidValue(&'static str),

    #[error("Property {0} is required but missing.")]
    MissingRequiredProperty(&'static str),

    #[error("Property {0} is invalid.")]
    InvalidProperty(&'static str),

    #[error("Property {0} cannot coexist with {1}.")]
    CannotCoexist(&'static str, &'static str),

    #[error("Property {0} requires {1}.")]
    Requires(&'static str, &'static str)
}


pub fn parse(rule: &str) -> Result<RecurrenceRule, RRuleParseError>
{
    let props = rule
        .split(';')
        .map(|x| {
            let sides: Vec<&str> = x.split('=').collect();

            let prop_name = *sides.get(0).unwrap();
            let prop_value = *sides.get(1).unwrap();

            (prop_name, prop_value)
        })
        .collect::<HashMap<&str, &str>>();

    // FREQ

    let frequency;
    if let Some(freq) = props.get("FREQ")
    {
        frequency = match *freq
        {
            "DAILY" => RecurrenceFreq::Daily,
            "WEEKLY" => RecurrenceFreq::Weekly,
            "MONTHLY" => RecurrenceFreq::Monthly,
            "YEARLY" => RecurrenceFreq::Yearly,
            _ => return Err(RRuleParseError::InvalidValue("FREQ"))
        }
    }
    else
    {
        return Err(RRuleParseError::MissingRequiredProperty("FREQ"))
    }



    // INTERVAL

    let interval: i32 = props.get("INTERVAL")
        .map(|x| x.parse::<i32>())
        .unwrap_or(Ok(1))
        .map_err(|_e| RRuleParseError::InvalidValue("INTERVAL"))?;

    if interval.is_negative()
    {
        return Err(RRuleParseError::InvalidValue("INTERVAL"));
    }



    // LIMIT

    let limit: RecurrenceLimit;
    if let Some(until) = props.get("UNTIL")
    {
        if props.get("COUNT").is_some()
        {
            return Err(RRuleParseError::DuplicateProperty("UNTIL and COUNT"))
        }

        let date = parse_date(until)
            .map_err(|_e| RRuleParseError::InvalidValue("UNTIL"))?;

        limit = RecurrenceLimit::Date(date);
    }
    else if let Some(count) = props.get("COUNT")
    {
        limit = RecurrenceLimit::Count(count.parse::<u32>().map_err(|_| RRuleParseError::InvalidValue("COUNT"))?)
    }
    else
    {
        limit = RecurrenceLimit::Indefinite
    }



    // BYDAY
    let by_day: Option<Vec<Weekday>> = props.get("BYDAY")
        .map(|x| parse_list(x,
            |val| match val {
                "MO" => Ok(Weekday::Mon),
                "TU" => Ok(Weekday::Tue),
                "WE" => Ok(Weekday::Wed),
                "TH" => Ok(Weekday::Thu),
                "FR" => Ok(Weekday::Fri),
                "SA" => Ok(Weekday::Sat),
                "SU" => Ok(Weekday::Sun),
                _ =>    Err(RRuleParseError::InvalidValue("BYDAY"))
            })
        )
        .transpose()?;


    // BYMONTH
    let by_month: Option<Vec<Month>> = parse_number_list(
        &props,
        "BYMONTH",
        &[&validate_range(1, 12, false)]
    )?
    .map(|vec| vec
        .into_iter()
        .map(|x| Month::from_i32(x).unwrap())
        .collect::<Vec<Month>>()
    );


    // BYYEARDAY
    let by_year_day: Option<Vec<i32>> = parse_number_list(
        &props,
        "BYYEARDAY",
        &[&validate_range(-366, 366, false)]
    )?;

    if by_year_day.is_some() && matches!(frequency, RecurrenceFreq::Daily | RecurrenceFreq::Weekly | RecurrenceFreq::Monthly)
    {
        return Err(RRuleParseError::CannotCoexist("BYYEARDAY", "FREQ=DAILY or FREQ=WEEKLY or FREQ=MONTHLY"));
    }

    // BYMONTHDAY
    let by_month_day: Option<Vec<i32>> = parse_number_list(
        &props,
        "BYMONTHDAY",
        &[&validate_range(-31, 31, false)]
    )?;

    if by_month_day.is_some() && frequency == RecurrenceFreq::Weekly
    {
        return Err(RRuleParseError::CannotCoexist("BYMONTHDAY", "FREQ=WEEKLY"));
    }


    // BYWEEKNO
    let by_week_no: Option<Vec<i32>> = parse_number_list(
        &props,
        "BYWEEKNO",
        &[&validate_range(-53, 53, false)]
    )?;

    if by_week_no.is_some() && frequency != RecurrenceFreq::Yearly
    {
        return Err(RRuleParseError::Requires("BYWEEKNO", "FREQ=YEARLY"));
    }

    // BYSETPOS
    let by_set_pos: Option<Vec<i32>> = parse_number_list(
        &props,
        "BYSETPOS",
        &[&validate_range(-366, 366, false)]
    )?;

    if by_set_pos.is_some()
        && by_day.is_none()
        && by_month.is_none()
        && by_year_day.is_none()
        && by_month_day.is_none()
        && by_week_no.is_none()
    {
        return Err(RRuleParseError::Requires("BYSETPOS", "BYDAY or BYMONTH or BYYEARDAY or BYMONTHDAY or BYWEEKNO"));
    }


    let ret_val = RecurrenceRule {
        frequency,
        interval,
        limit,
        by_day,
        by_month,
        by_year_day,
        by_month_day,
        by_week_no,
        by_set_pos,
    };

    Ok(ret_val)
}

fn parse_date(value: &str) -> ParseResult<NaiveDate>
{
    NaiveDate::parse_from_str(value, "%Y%m%d")
}

fn parse_number_list(props: &HashMap<&str, &str>, prop_name: &'static str, validators: &[&dyn Fn(i32) -> bool]) -> Result<Option<Vec<i32>>, RRuleParseError>
{
    if let Some(prop_value) = props.get(prop_name)
    {
        let list = parse_list(prop_value,
            |item_str|
            {
                item_str.parse::<i32>()
                    .map_err(|_| RRuleParseError::InvalidValue(prop_name))
                    .and_then(
                        |val|
                        {
                            let is_valid = validators
                                .iter()
                                .fold(true, |is_valid, validator| is_valid && validator(val));

                            if is_valid
                            {
                                Ok(val)
                            }
                            else
                            {
                                Err(RRuleParseError::InvalidValue(prop_name))
                            }
                        }
                    )
            }
        )?;

        Ok(Some(list))
    }
    else
    {
        Ok(None)
    }
}

fn validate_range(min: i32, max: i32, zero: bool) -> impl Fn(i32) -> bool
{
    move |x| {
        x >= min && x <= max && (zero || x != 0)
    }
}

fn parse_list<T, F>(value: &str, item_parser: F) -> Result<Vec<T>, RRuleParseError>
    where F: Fn(&str) -> Result<T, RRuleParseError>
{
    value.split(',')
        .map(|val| item_parser(val))
        .collect::<Result<Vec<T>, RRuleParseError>>()
}

#[cfg(test)]
mod test
{
    use super::{RecurrenceRule, RecurrenceFreq, RecurrenceLimit};
    
    use chrono::{NaiveDate, Month, Weekday};
    

    impl Default for RecurrenceRule
    {
        fn default() -> Self
        {
            RecurrenceRule {
                frequency: RecurrenceFreq::Daily,
                interval: 1,
                limit: RecurrenceLimit::Indefinite,
                by_month: None,
                by_week_no: None,
                by_year_day: None,
                by_month_day: None,
                by_day: None,
                by_set_pos: None,
            }
        }
    }

    #[test]
    fn parse_freq()
    {
        let result = super::parse("FREQ=WEEKLY").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Indefinite,
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_count()
    {
        let result = super::parse("FREQ=WEEKLY;COUNT=10").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Count(10),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_until()
    {
        let result = super::parse("FREQ=WEEKLY;UNTIL=20200101").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Date(NaiveDate::from_ymd(2020, 1, 1)),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_month()
    {
        let result = super::parse("FREQ=WEEKLY;BYMONTH=1,4,6").unwrap();


        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            by_month: Some(vec![Month::January, Month::April, Month::June]),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_month_day()
    {
        let result = super::parse("FREQ=MONTHLY;BYMONTHDAY=1,4,6,31,-1,9").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Monthly,
            by_month_day: Some(vec![1, 4, 6, 31, -1, 9]),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_day()
    {
        let result = super::parse("FREQ=MONTHLY;BYDAY=MO,TU,WE,SA").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Monthly,
            by_day: Some(vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Sat]),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_week_no()
    {
        let result = super::parse("FREQ=YEARLY;BYWEEKNO=-30,15,2").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Yearly,
            by_week_no: Some(vec![-30, 15, 2]),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_year_day()
    {
        let result = super::parse("FREQ=YEARLY;BYYEARDAY=-54,344,25").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Yearly,
            by_year_day: Some(vec![-54, 344, 25]),
            ..RecurrenceRule::default()
        });
    }

    #[test]
    fn parse_by_set_pos()
    {
        let result = super::parse("FREQ=MONTHLY;BYDAY=SA;BYSETPOS=-1,3,4").unwrap();

        assert_eq!(result, RecurrenceRule {
            frequency: RecurrenceFreq::Monthly,
            by_day: Some(vec![Weekday::Sat]),
            by_set_pos: Some(vec![-1, 3, 4]),
            ..RecurrenceRule::default()
        });
    }
}