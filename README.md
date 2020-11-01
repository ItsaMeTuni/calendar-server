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

This is **NOT** aimed at being a personal calendar server, this is meant to be a calendar backend used by your application's backend.

**Disclaimer:** I'm a hobbyist, which means I'm not an expert in anything. Some mentoring would be helpful.

## Documentation

You can read the documentation of the project [here](./docs).

## How to develop

Clone the repo and run `docker-compose up -d`, this will start the postgres container. Then you can do `cargo run` to run the server or run it from your IDE of preference.

## What Calendar Server does **NOT** support

- `HOURLY`, `SECONDLY` and `MINUTELY` RRULE frequencies.
- `BYHOUR`, `BYSECOND` and `BYMINUTE` RRULE constraints.
- The `WKST` RRULE part. Weeks always start on mondays.