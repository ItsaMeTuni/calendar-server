use rocket::{Request, request::Outcome, State};

use rocket::request::FromRequest;


use crate::configs::Configs;




/// Common query parameters most routes support, like
/// the `offset` and `limit`
pub struct CommonQueryParams
{
    offset: u32,
    page_size: u32,
}

impl CommonQueryParams
{
    /// Returns the maximum amount of elements (i.e. array items) that can be present
    /// in a response, akin to the SQL LIMIT clause..
    /// This will return the `limit` parameter if it is specified and smaller than
    /// the configured page size. Otherwise this falls back to the configured page size.
    ///
    /// The value returned by this should generally be used as a LIMIT clause in
    /// SQL queries.
    ///
    /// This returns an i64 but "under the hood" it's a u32, this is done
    /// so that you don't have to do an `as i64` cast to pass this as a parameter
    /// to SQL queries.
    pub fn page_size(&self) -> i64 { self.page_size as i64 }

    /// Get the `offset` parameter. This is the amount of elements (i.e. array items)
    /// that should be "skipped" from a response, akin to the SQL OFFSET clause.
    ///
    /// The value returned by this should generally be used as an OFFSET clause in
    /// SQL queries.
    ///
    /// This returns an i64 but "under the hood" it's a u32, this is done
    /// so that you don't have to do an `as i64` cast to pass this as a parameter
    /// to SQL queries.
    pub fn offset(&self) -> i64 { self.offset as i64 }
}

impl<'a, 'r> FromRequest<'a, 'r> for CommonQueryParams
{
    type Error = ();

    fn from_request(request: &Request) -> Outcome<Self, Self::Error>
    {
        match request.guard::<State<Configs>>()
        {
            Outcome::Success(configs) =>
            {
                let offset: Option<u32> = request.get_query_value("offset").transpose().unwrap_or(None);
                let limit_param: Option<u32> = request.get_query_value("limit").transpose().unwrap_or(None);

                let limit: u32;
                if limit_param.is_some() && limit_param.unwrap() < configs.get_page_size()
                {
                    limit = limit_param.unwrap();
                }
                else
                {
                    limit = configs.get_page_size();
                }

                Outcome::Success(
                    CommonQueryParams {
                        offset: offset.unwrap_or(0),
                        page_size: limit,
                    }
                )
            },
            Outcome::Forward(f) => Outcome::Forward(f),
            Outcome::Failure(e) => Outcome::Failure(e),
        }


    }
}