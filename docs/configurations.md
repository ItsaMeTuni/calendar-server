
# Configs

This page describes all configuration options for the calendar server.

### Page size
<a name="page-size"></a>

- **Type:** Integer > 0

- **Default:** 1000

- **Description:** Maximum amount of resources returned in a single request. E.g. if the calendar has 1200 events and you make a request to `GET /calendars/<calendar-id>/events` and the page size is 1000, only 1000 events will be returned. To get the last 200 events you should use an offset parameter (or equivalent) with the value of 1000.