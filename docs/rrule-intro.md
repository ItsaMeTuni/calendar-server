# Brief RRULE introduction

The RRULE is a string format used to describe event recurrence. Here are a few examples:

**`FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,TU`:** An event that happens weekly on mondays and tuesdays.

**`FREQ=MONTHLY;INTERVAL=3;BYMONTHDAY=10`:** An event that happens every 3 months on the 10th of the month.

**`FREQ=WEEKLY`:** An event that happens every week. `BYDAY` is inferred from the event's `start_date` and interval defaults to `1`.


**Possible values for `FREQ`:** `YEARLY`, `MONTHLY`, `WEEKLY`, `DAILY`. Hourly, minutely and secondly are not supported by our implementation.

**Possible values for `BYDAY`:** `MO`, `TU`, `WE`, `TH`, `FR`, `SA`, `SU`.

**Possible values for `BYMONTHDAY`:** 1 to 31. Caveat: if the month does not have the `BYMONTHDAY` day (like Feb 30), the event will never occur on that month and that day.

There are many more options and configurations. I recommend that you read the RFC 5545 to learn more about it.
