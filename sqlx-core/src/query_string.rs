use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A SQL string that is safe to execute on a database connection.
///
/// A "safe" query string is one that is unlikely to contain a [SQL injection vulnerability][injection].
///
/// In practice, this means a string type that is unlikely to contain dynamic data or user input.
///
/// This is designed to act as a speedbump against naively using `format!()` to add dynamic data
/// or user input to a query, which is a classic vector for SQL injection as SQLx does not
/// provide any sort of escaping or sanitization (which would have to be specially implemented
/// for each database flavor/locale).
///
/// The recommended way to incorporate dynamic data or user input in a query is to use
/// bind parameters, which requires the query to execute as a prepared statement.
/// However, bind parameters don't work for all situations; they cannot affect the SQL itself.
/// See [`query()`] for details.
///
/// `&'static str` is the only string type that satisfies the requirements of this trait
/// (ignoring [`String::leak()`] which has niche use-cases) and so is the only string type that
/// natively implements this trait by default.
///
/// For other string types, use [`AssertQuerySafe`] to assert this property.
/// This is the only intended way to pass an owned `String` to [`query()`] and its related functions.
///
/// This trait and `AssertQuerySafe` are intentionally analogous to [`std::panic::UnwindSafe`] and
/// [`std::panic::AssertUnwindSafe`].
///
/// [injection]: https://en.wikipedia.org/wiki/SQL_injection
/// [`query()`]: crate::query::query
pub trait QuerySafeStr<'a> {
    /// Wrap `self` as a [`QueryString`].
    fn into_query_string(self) -> QueryString<'a>;
}

impl QuerySafeStr<'static> for &'static str {
    #[inline]

    fn into_query_string(self) -> QueryString<'static> {
        QueryString(Repr::Static(self))
    }
}

/// Assert that a query string is safe to execute on a database connection.
///
/// Note that `&'static str` implements [`QuerySafeStr`] directly and so does not need to be wrapped
/// with this type.
///
/// # Warning: use at your own risk!
/// Using this API means that **you** have made sure that the string contents do not contain a
/// [SQL injection vulnerability][injection]. It means that, if the string was constructed
/// dynamically, and/or from user input, you have taken care to sanitize the input yourself.
///
/// The maintainers of SQLx take no responsibility for any data leaks or loss resulting from the use
/// of this API. SQLx does not provide any sort of escaping or sanitization for queries.
///
/// The recommended way to incorporate dynamic data or user input in a query is to use
/// bind parameters, which requires the query to execute as a prepared statement.
/// However, bind parameters don't work for all situations; they cannot affect the SQL itself.
/// See [`query()`] for details.
///
/// [injection]: https://en.wikipedia.org/wiki/SQL_injection
/// [`query()`]: crate::query::query
///
/// ## Example: Query String from `format!()`
/// A safe use of this API would be to change the query in one of a fixed number of ways:
///
/// ```rust,no_run
/// # async fn example3() -> sqlx::Result<()> {
/// use time::OffsetDateTime;
/// use uuid::Uuid;
///
/// use sqlx::{PgConnection, Postgres};
/// use sqlx::postgres::{PgArguments, PgRow};
/// use sqlx::query::{AssertQuerySafe, Query};
///
/// use std::fmt::Write;
///
/// enum OrderBy {
///     Score,
///     CreatedAt,
///     UpdatedAt,
/// }
///
/// fn posts_query(order_by: OrderBy, order_inverted: bool, limit: Option<i64>) -> Query<'static, Postgres, PgArguments> {
///     // Because we're hardcoding a small set of column names,
///     // it's impossible to accidentally introduce a SQL injection vulnerability this way.
///     //
///     // Any dynamic changes to the SQL string should ideally always pass through
///     // some sort of validation layer like this.
///     let order_by = match order_by {
///         OrderBy::Score => "score",
///         OrderBy::CreatedAt => "created_at",
///         OrderBy::UpdatedAt => "updated_at",
///     };
///
///     let sorting = if order_inverted {
///         // Assuming `updated_at` is the only nullable column,
///         // this would push posts without an `updated_at` to the front.
///         "ASC NULLS FIRST"
///     } else {
///         "DESC NULLS LAST"
///     };
///
///     // The expression for `LIMIT` can actually be a bind parameter,
///     // so we don't need to change that in the string content itself.
///     //
///     // Passing `NULL` for a limit (in the case `limit = None`) is the same as setting no limit.
///     let mut query = format!("SELECT * FROM posts ORDER BY {order_by} {sorting} LIMIT $1");
///
///     sqlx::query(AssertQuerySafe(query)).bind(limit)
/// }
///
///
/// let mut conn: PgConnection = unimplemented!("get a PgConnection");
/// let posts = posts_query(OrderBy::CreatedAt, false, Some(50))
///     .fetch_all(&mut conn)
///     .await?;
///
/// # Ok(())
/// # }
/// ```
pub struct AssertQuerySafe<T>(pub T);

/// Covers the following types (non-exhaustive):
/// * `&'a str`
/// * `&'a String`
/// * Nested references of the above
///
/// Note that this will require [`AssertQuerySafe::into_static()`] to copy the string.
// (Cannot be `for AssertQuerySafe<T> where T: AsRef<str>` as that would overlap with the others.)
impl<'a, T> QuerySafeStr<'a> for AssertQuerySafe<&'a T>
where
    T: AsRef<str>,
{
    #[inline]
    fn into_query_string(self) -> QueryString<'a> {
        QueryString(Repr::Slice(self.0.as_ref()))
    }
}

impl QuerySafeStr<'static> for AssertQuerySafe<String> {
    #[inline]
    fn into_query_string(self) -> QueryString<'static> {
        QueryString(Repr::Owned(self.0))
    }
}

impl QuerySafeStr<'static> for AssertQuerySafe<Box<str>> {
    #[inline]
    fn into_query_string(self) -> QueryString<'static> {
        QueryString(Repr::Boxed(self.0))
    }
}

// Note: this is not implemented for `Rc<str>` because it would make `QueryString: !Send`.
impl QuerySafeStr<'static> for AssertQuerySafe<Arc<str>> {
    #[inline]
    fn into_query_string(self) -> QueryString<'static> {
        QueryString(Repr::Arced(self.0))
    }
}

/// A SQL string that is ready to execute on a database connection.
///
/// This is essentially `Cow<'a, str>` but which can be constructed from additional types
/// without copying.
///
/// This type also asserts the property that the query string is safe to execute against a database.
/// See [`QuerySafeStr`] for details.
///
/// This type is not designed to be manually constructable.
#[derive(Clone, Debug)]
pub struct QueryString<'a>(Repr<'a>);

#[derive(Clone, Debug)]
enum Repr<'a> {
    Slice(&'a str),
    // We need a variant to memoize when we already have a static string, so we don't copy it.
    Static(&'static str),
    // Newer Rust versions are actually able to use an additional niche provided by
    // the capacity field of `RawVec` and thus this type, even with a `String` variant,
    // is still only 3 `usize`s wide.
    //
    // This is because the capacity field is now a newtype that specifies
    // a valid range of `0 ..= isize::MAX`, so the bitpatterns `80...` to `FF...` are now fair game
    // for niche optimization:
    // https://github.com/rust-lang/rust/blob/6c90ac8d8f4489472720fce03c338cd5d0977f33/library/alloc/src/raw_vec.rs#L39
    //
    // It'd still be 3 words without this variant because DSTs don't provide this niche,
    // so it would have to reserve space for the discriminant plus padding for the alignment.
    Owned(String),
    Boxed(Box<str>),
    Arced(Arc<str>),
}

impl<'a> QuerySafeStr<'a> for QueryString<'a> {
    #[inline]
    fn into_query_string(self) -> QueryString<'a> {
        self
    }
}

impl QueryString<'_> {
    /// Erase the borrow in `self` by copying the string to a new allocation if necessary.
    ///
    /// This copies the string if `self` was constructed from `AssertQuerySafe<&'a str>`
    /// or another string reference type.
    ///
    /// In all other cases, this is a no-op.
    #[inline]
    pub fn into_static(self) -> QueryString<'static> {
        QueryString(match self.0 {
            Repr::Slice(s) => Repr::Boxed(s.into()),
            Repr::Static(s) => Repr::Static(s),
            Repr::Owned(s) => Repr::Owned(s),
            Repr::Boxed(s) => Repr::Boxed(s),
            Repr::Arced(s) => Repr::Arced(s),
        })
    }

    /// Borrow the inner query string.
    #[inline]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Repr::Slice(s) => s,
            Repr::Static(s) => s,
            Repr::Owned(s) => s,
            Repr::Boxed(s) => s,
            Repr::Arced(s) => s,
        }
    }
}

impl AsRef<str> for QueryString<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for QueryString<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<T> PartialEq<T> for QueryString<'_>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl Eq for QueryString<'_> {}

impl Hash for QueryString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}
