# Common stuff on the API

## Common parameters

These parameters are very common in the API routes, so we describe them all in one place. However, every route will state whether or not it supports any of the parameters described here.

Parameters described here are guaranteed to behave the same on all routes that use them.

### The `offset` parameter
<a name="param-offset"></a>

- **Type:** number (>= 0)
- **Description:** Skip this many rows of the result. Useful when the query results in more items than the [page size](./configurations.md#page-size) allows for.