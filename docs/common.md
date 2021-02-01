# Common stuff on the API

## Common headers

### The `Authorization` header
<a name="header-authorization"></a>

All (or almost all) routes of the API require an `Authorization` header with an API key. You can obtain
an API key by accessing the database and running the following queries:

```sql
INSERT INTO api_keys(scopes) VALUES (array['WRITE']);
SELECT api_key FROM api_keys;
```

For information on scopes and api key permissions take a look [here](./scopes.md).

## Common parameters

These parameters are very common in the API routes, so we describe them all in one place. However, every route will state whether or not it supports any of the parameters described here.

Parameters described here are guaranteed to behave the same on all routes that use them.

### The `offset` parameter
<a name="param-offset"></a>

- **Type:** number (>= 0)
- **Description:** Skip this many rows of the result. Useful when the query results in more items than the [page size](./configurations.md#page-size) allows for.