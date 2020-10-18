This project aims to develop a fast and easy to setup calendar backend that complies with RFC 5545 and exposes a RESTful API for manipulating events and calendars.

**Why?** Because Google Calendar's API, which seems to be the best option for a calendar backend at the moment, sucks. And because it's fun :)

**What about X?** Other calendar backends I've found around the internet are either old, run on php, or have horrible documentation.

It's written in Rust and stores data in Postgres.

This is **NOT** aimed at being a personal calendar server, this is meant to be a calendar backend, not directly exposed to the internet. Initially this won't have any sort of security, users, or ACLs, but those are some features I plan on adding on the future.

**Note:** I suck, I don't have a lot of knowledge on security, databases, calendar stuff, or even rust. I know a bit of each, but I'm not an expert by any means, so please if there's anything I can improve please let me know :)


## How do develop

Clone the repo and run `docker-compose up -d`, this will start the postgres container. Then you can do `cargo run` to run the server or run it from your IDE of preference.