# Calendar

**IMPORTANT:** You need an API key to execute most requests. Take a look at [common](./common.md#header-authorization).

## The Calendar object

Properties:
- `id` (integer): Id of the calendar
- `last_modified` (date-time string): Timestamp of the last time the calendar was modified. Does not change when it's events are modified.

**Note:** Currently the `last_modified` property isn't very useful since there aren't any properties we can change on the calendar yet.

## Actions

### List calendars

`GET /api/calendars`

Returns an array of Calendar objects.

#### Optional parameters 

Parameter name | Type | Description
-|-|-
`offset` | number (>= 0) | [Offset parameter](./common.md#param-offset)

### Get calendar

`GET /api/calendars/<calendar-id>`

Returns a Calendar object.

### Insert calendar

`POST /api/calendars`

Expects a Calendar object without the `id` field.

# Event

## The event object

Properties:
- `id` (integer): Id of the event
- `parent_id` (integer): Id of the event that originated this one from its recurrence rule. More on this later.
- `start_date` (date string): The start date of the event
- `start_time` (time string, optional): The start time of the event
- `end_date` (date string): The end date of the event
- `end_time` (time string, optional): The end time of the event
- `recurrence` (Recurrence Object, optional): The recurrence of the event

### Constraints

- If `start_time` is set, `end_time` must also be set and vice-versa.
- Start date/date+time must be smaller than end date/date+time.

### About the `parent_id`

Imagine there's a recurrent event of id 5 that happens every week on wednesdays (`FREQ=WEEKLY;INTERVAL=1;BYDAY=WE`) at 15:00 and starts on `2020-01-01`.

Now you want to re-schedule the _event instance_ of `2020-01-08` to happen at 16PM (one hour later), to do that you make the following request:

```http
PUT /calendars/1/events/5/instances/2020-01-08

{
    "start_time": "16:00",
    "end_time": "17:00"
}
```

And it returns something like

```http
201 CREATED
Location: /calendars/1/events/6

{
    "id": 6,
    "parent_id": 5,
    "start_date": "2020-01-08",
    "start_time": "16:00",
    "end_date": "2020-01-08",
    "end_time": "17:00"

    // other fields...
}
```

This will add `2020-01-08` to event 5's recurrence exdates property and create a new event that starts at `2020-01-08T16:00`. Notice the `parent_id` property that is `5`, which is the if of the event that "originated" this one.

This is useful when cascading some property changes from the parent event to the child event. If we want to change the `start_time` of the parent event and all of its children to `14:00`, we can make one request to update the parent event, then another to query all child events, and then other requests to update the children.

## The Recurrence object

Properties:
- `rrule` (RFC 5545 RRULE string): An RRULE as defined in RFC 5545
- `exdates` (date string array): The dates on which this event does not happen
- `rdates` (date string array): The extra dates on which this event happens

### Constraints

- `rrule` must be an RRULE as defined in RFC 5545 (if you're not familiar with it there's a little introduction [here](./rrule-intro.md)).

## Actions

### List events

`GET /calendars/<calendar-id>/events`

Returns an array of Event objects. Does **not** return event instances, if you want that take a look [here](#get-event-instances).

#### Optional parameters 

Parameter name | Type | Description
-|-|-
`since` | string (ISO date or ISO date-time) | Events start date lowe bound (inclusive). Only return events with a start date bigger than this value.
`until` | string (ISO date or ISO date-time) | Events end date upper bound (inclusive). Only return events with an end date smaller than this value.
`offset` | number (>= 0) | [Offset parameter](./common.md#param-offset)

### Get event

`GET /calendars/<calendar-id>/events/<event-id>`

Returns an Event object.

### Insert event

`POST /calendars/<calendar-id>/events`

Expects an Event object without id.

### Update event

`PUT /calendars/<calendar-id>/events/<event-id>`

Expects an Event object in which all fields are optional. If the event's `id` field is specified it **must** be the same as `<event-id>`. All fields that are not specified in the request's body are left unchanged.


### Get event instances
<a name="get-event-instances"></a>

`GET /calendars/<calendar-id>/events/<event-id>/instances`

Returns an array of Event objects that are _event instances_ of the event. Returns 404 if the event is not recurring.

#### Required parameters

Parameter name | Type | Description
-|-|-
`since` | string (ISO date) | Query event instances that happen at or after this date.
`until` | string (ISO date) | Query event instances that happen at or before this date.

#### Optional parameters 

Parameter name | Type | Description
-|-|-
`offset` | number (>= 0) | [Offset parameter](./common.md#param-offset)

### Check for changes

`GET /calendars/<calendar-id>/events/changes`

Query for events that were modified at or after the date/date-time specified in the `since` parameter. I.e. query for events with `last_modified` >= `since` param.

#### Required parameters

Parameter name | Type | Description
-|-|-
`since` | string (ISO date or ISO date-time) | Query event instances that were modified at or after this date/date-time.

#### Optional parameters

Parameter name | Type | Description
-|-|-
`offset` | number (>= 0) | [Offset parameter](./common.md#param-offset)