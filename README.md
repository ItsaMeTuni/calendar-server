**VERY WIP**

**What?** A self-hosted calendaring backend alternative to Google Calendar. It exposes a RESTful API for manipulating events and integrates with FusionAuth to provide authentication.

**Features:**
- Recurrent events (using the RFC 5545's RRULE syntax)
- Multiple calendars
- ACLs
- Webhooks for monitoring changes to calendars and events
- Easy-to-use API
- Self-hosted
- Easy setup
- Integraton with FusionAuth to provide authentication

**Why?** Because even though Google Calendar's API sucks, it still seemed to be the best option for a calendar backend until now. This is a good, self-hosted, free and open-source alternative to it.

**How?** It's written in Rust (using the Rocket framework), stores everything in PostgreSQL and uses FusionAuth for authentication.

**What about X?** Other calendar backends I've found around the internet are either old, run on php, or have horrible documentation.

This is **NOT** aimed at being a personal calendar server, this is meant to be a calendar backend used by your application's frontend/backend.

**Disclaimer:** I'm a hobbyist, which means I'm not an expert in anything. Some mentoring would be helpful.

## Examples

### Create an event

This request creates a weekly event that happens from 12:00 to 13:00, starting on Feb 1st 2021:
```http
POST /api/calendars/bf10b852-bbcc-43be-93c8-3c236e764247/events HTTP/1.1
Host: localhost:8000
Content-Type: application/json
Authorization: 7d104549-8953-459b-a69b-8ef268a47170

{
  "start_date": "2021-02-01",
  "start_time": "12:00",
  "end_date": "2021-02-01",
  "end_time": "13:00",
  "recurrence": {
    "rrule": "FREQ=WEEKLY",
    "rdates": [],
  	"exdates": []
  }
}
```

The response is a 200 OK with the following body:
```http
{
    "id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
    "parent_id": null,
    "start_date": "2021-02-01",
    "start_time": "12:00",
    "end_date": "2021-02-01",
    "end_time": "13:00",
    "recurrence": {
        "rrule": "FREQ=WEEKLY",
        "exdates": [],
        "rdates": []
    },
    "last_modified": "2021-02-01T18:20"
}
```

### List an event's instances

This request lists the recurrence instances between 2021-02-01 and 2021-03-01 from the event we just created:
```http
GET /api/calendars/bf10b852-bbcc-43be-93c8-3c236e764247/events/cb3c0e50-7cb1-464b-8db5-af712f79a4e8/instances?since=2021-02-01&amp; until=2021-03-01 HTTP/1.1
Host: localhost:8000
Authorization: 7d104549-8953-459b-a69b-8ef268a47170
```

Here is the body of the response:
```http
[
    {
        "id": null,
        "parent_id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
        "start_date": "2021-02-01",
        "start_time": "12:00",
        "end_date": "2021-02-01",
        "end_time": "13:00",
        "recurrence": null,
        "last_modified": null
    },
    {
        "id": null,
        "parent_id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
        "start_date": "2021-02-08",
        "start_time": "12:00",
        "end_date": "2021-02-08",
        "end_time": "13:00",
        "recurrence": null,
        "last_modified": null
    },
    {
        "id": null,
        "parent_id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
        "start_date": "2021-02-15",
        "start_time": "12:00",
        "end_date": "2021-02-15",
        "end_time": "13:00",
        "recurrence": null,
        "last_modified": null
    },
    {
        "id": null,
        "parent_id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
        "start_date": "2021-02-22",
        "start_time": "12:00",
        "end_date": "2021-02-22",
        "end_time": "13:00",
        "recurrence": null,
        "last_modified": null
    },
    {
        "id": null,
        "parent_id": "cb3c0e50-7cb1-464b-8db5-af712f79a4e8",
        "start_date": "2021-03-01",
        "start_time": "12:00",
        "end_date": "2021-03-01",
        "end_time": "13:00",
        "recurrence": null,
        "last_modified": null
    }
]
```

## Documentation

You can read the documentation of the project [here](./docs).

## How to develop

1. Clone the repo
2. Create a `.env` file at the root of the repository with the following contents
```
POSTGRES_PASSWORD=changeme
POSTGRES_USER=calendarserver

FA_DATABASE_PASSWORD=changeme
FA_FUSIONAUTH_MEMORY=512M

DB_ADDR=localhost:6789
```
3. Run `docker-compose up -d`, this will start the postgres container.
4. Run `cargo run` to run the server or run it from your IDE of preference.
5. Run `psql -h localhost -p 6789 -U calendarserver` and then type the password in the env variable `POSTGRES_PASSWORD`.
6. Run `INSERT INTO api_keys(scopes) VALUES (array['SUPER']);` to create an api key.
7. Run `SELECT api_key FROM api_keys;` and copy the API key, you'll put it in the `Authorization` header of
each request you make to the API.

## What Calendar Server does **NOT** support

- `HOURLY`, `SECONDLY` and `MINUTELY` RRULE frequencies.
- `BYHOUR`, `BYSECOND` and `BYMINUTE` RRULE constraints.
- The `WKST` RRULE part. Weeks always start on mondays.