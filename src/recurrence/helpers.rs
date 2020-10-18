use chrono::{Datelike};

pub trait NaiveDateHelpers: Datelike
{
    /// Get day of the year. E.g. for 2020-01-01 this function
    /// will return 1, for 2020-02-15 this will return 46, you
    /// get the picture.
    ///
    /// Accounts for leap years (2020-03-01 will return 61, 2021-03-01 will return 60).
    fn year_day(&self) -> u32
    {
        let is_leap_year = self.year() % 4 == 0 && (self.year() % 100 != 0 || self.year() % 400 == 0);

        let mut day_count = self.day();

        for month in 1..self.month()
        {
            if month == 2
            {
                if is_leap_year
                {
                    day_count += 29;
                }
                else
                {
                    day_count += 28;
                }
            }
            else if month < 8
            {
                // Odd months: January, March, May, July
                if month % 2 != 0
                {
                    day_count += 31;
                }
                else // Even months: April, June (February was caught in the previous if)
                {
                    day_count += 30;
                }
            }
            else
            {
                // Odd months: September, November
                if month % 2 != 0
                {
                    day_count += 30;
                }
                else // Even months: August, October, December
                {
                    day_count += 31;
                }
            }
        }

        day_count
    }
}

impl<T: Datelike> NaiveDateHelpers for T {}

#[cfg(test)]
mod test
{
    use chrono::NaiveDate;
    use super::NaiveDateHelpers;

    #[test]
    fn year_day()
    {
        assert_eq!(NaiveDate::from_ymd(2020, 02, 15).year_day(), 46);
        assert_eq!(NaiveDate::from_ymd(2020, 04, 04).year_day(), 95);
        assert_eq!(NaiveDate::from_ymd(2020, 12, 31).year_day(), 366);
        assert_eq!(NaiveDate::from_ymd(2019, 12, 31).year_day(), 365);
        assert_eq!(NaiveDate::from_ymd(2021, 07, 30).year_day(), 211);
    }
}