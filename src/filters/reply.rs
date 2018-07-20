//! Reply Filters
//!
//! These "filters" behave a little differently than the rest. Instead of
//! being used directly on requests, these filters "wrap" other filters.
//!
//!
//! ## Wrapping a `Filter` (`with`)
//!
//! ```
//! use warp::Filter;
//!
//! let with_server = warp::reply::with::header("server", "warp");
//!
//! let route = warp::any()
//!     .map(warp::reply)
//!     .with(with_server);
//! ```
//!
//! Wrapping allows adding in conditional logic *before* the request enters
//! the inner filter (though the `with::header` wrapper does not).

use http::header::{HeaderName, HeaderValue};
use http::HttpTryFrom;

use ::filter::{Filter, Map, One, WrapSealed};
use ::reply::Reply;
use self::sealed::{WithHeader_, WithDefaultHeader_};

/// Wrap a [`Filter`](::Filter) that adds a header to the reply.
///
/// # Example
///
/// ```
/// use warp::Filter;
///
/// // Always set `foo: bar` header.
/// let route = warp::any()
///     .map(warp::reply)
///     .with(warp::reply::with::header("foo", "bar"));
/// ```
pub fn header<K, V>(name: K, value: V) -> WithHeader
where
    HeaderName: HttpTryFrom<K>,
    HeaderValue: HttpTryFrom<V>,
{
    let (name, value) = assert_name_and_value(name, value);
    WithHeader {
        name,
        value,
    }
}

// pub fn headers?

/// Wrap a [`Filter`](::Filter) that adds a header to the reply, if they
/// aren't already set.
///
/// # Example
///
/// ```
/// use warp::Filter;
///
/// // Set `server: warp` if not already set.
/// let route = warp::any()
///     .map(warp::reply)
///     .with(warp::reply::with::default_header("server", "warp"));
/// ```
pub fn default_header<K, V>(name: K, value: V) -> WithDefaultHeader
where
    HeaderName: HttpTryFrom<K>,
    HeaderValue: HttpTryFrom<V>,
{
    let (name, value) = assert_name_and_value(name, value);
    WithDefaultHeader {
        name,
        value,
    }
}

/// Wrap a `Filter` to always set a header.
#[derive(Clone, Debug)]
pub struct WithHeader {
    name: HeaderName,
    value: HeaderValue,
}

impl WithHeader {
    #[doc(hidden)]
    #[deprecated(note="use Filter::with(decorator) instead")]
    pub fn decorate<F, R>(&self, inner: F) -> Map<F, WithHeader_>
    where
        F: Filter<Extract=One<R>>,
        R: Reply,
    {
        inner.with(self)
    }
}

impl<F, R> WrapSealed<F> for WithHeader
where
    F: Filter<Extract=One<R>>,
    R: Reply,
{
    type Wrapped = Map<F, WithHeader_>;

    fn wrap(&self, filter: F) -> Self::Wrapped {
        let with = WithHeader_ {
            with: self.clone(),
        };
        filter.map(with)
    }
}


/// Wrap a `Filter` to set a header if it is not already set.
#[derive(Clone, Debug)]
pub struct WithDefaultHeader {
    name: HeaderName,
    value: HeaderValue,
}

impl WithDefaultHeader {
    #[doc(hidden)]
    #[deprecated(note="use Filter::with(decorator) instead")]
    pub fn decorate<F, R>(&self, inner: F) -> Map<F, WithDefaultHeader_>
    where
        F: Filter<Extract=One<R>>,
        R: Reply,
    {
        inner.with(self)
    }
}

impl<F, R> WrapSealed<F> for WithDefaultHeader
where
    F: Filter<Extract=One<R>>,
    R: Reply,
{
    type Wrapped = Map<F, WithDefaultHeader_>;

    fn wrap(&self, filter: F) -> Self::Wrapped {
        let with = WithDefaultHeader_ {
            with: self.clone(),
        };
        filter.map(with)
    }
}

fn assert_name_and_value<K, V>(name: K, value: V) -> (HeaderName, HeaderValue)
where
    HeaderName: HttpTryFrom<K>,
    HeaderValue: HttpTryFrom<V>,
{
    let name = <HeaderName as HttpTryFrom<K>>::try_from(name)
        .map_err(Into::into)
        .unwrap_or_else(|_| panic!("invalid header name"));

    let value = <HeaderValue as HttpTryFrom<V>>::try_from(value)
        .map_err(Into::into)
        .unwrap_or_else(|_| panic!("invalid header value"));

    (name, value)
}

mod sealed {
    use ::generic::{Func, One};
    use ::reply::{Reply, Reply_};
    use super::{WithHeader, WithDefaultHeader};

    #[derive(Clone)]
    #[allow(missing_debug_implementations)]
    pub struct WithHeader_ {
        pub(super) with: WithHeader,
    }

    impl<R: Reply> Func<One<R>> for WithHeader_ {
        type Output = Reply_;

        fn call(&self, args: One<R>) -> Self::Output {
            let mut resp = args.0.into_response();
            // Use "insert" to replace any set header...
            resp.headers_mut().insert(&self.with.name, self.with.value.clone());
            Reply_(resp)
        }
    }

    #[derive(Clone)]
    #[allow(missing_debug_implementations)]
    pub struct WithDefaultHeader_ {
        pub(super) with: WithDefaultHeader,
    }

    impl<R: Reply> Func<One<R>> for WithDefaultHeader_ {
        type Output = Reply_;

        fn call(&self, args: One<R>) -> Self::Output {
            let mut resp = args.0.into_response();
            resp
                .headers_mut()
                .entry(&self.with.name)
                .expect("parsed headername is always valid")
                .or_insert_with(|| self.with.value.clone());

            Reply_(resp)
        }
    }
}
