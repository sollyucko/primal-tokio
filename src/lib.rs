#![feature(async_closure)]
#![feature(type_ascription)]

use primal::Primes;
use tokio::stream::Stream;

pub fn primes_unbounded() -> impl Stream<Item = usize> {
    utils::spawn_stream_from_iterator(Primes::all())
}

mod utils {
    use tokio::future::poll_fn;
    use tokio::stream::Stream;
    use tokio::sync::mpsc;
    use tokio::task::spawn;

    mod error {
        use tokio::sync::mpsc::error::*;

        pub enum MpscSendError<T> {
            ClosedNoValue,
            Closed(T),
            Full(T),
            Timeout(T),
        }

        impl<T> From<ClosedError> for MpscSendError<T> {
            fn from(_err: ClosedError) -> Self {
                Self::ClosedNoValue
            }
        }

        impl<T> From<SendError<T>> for MpscSendError<T> {
            fn from(err: SendError<T>) -> Self {
                Self::Closed(err.0)
            }
        }

        impl<T> From<SendTimeoutError<T>> for MpscSendError<T> {
            fn from(err: SendTimeoutError<T>) -> Self {
                match err {
                    SendTimeoutError::Timeout(x) => Self::Timeout(x),
                    SendTimeoutError::Closed(x) => Self::Closed(x),
                }
            }
        }

        impl<T> From<TrySendError<T>> for MpscSendError<T> {
            fn from(err: TrySendError<T>) -> Self {
                match err {
                    TrySendError::Full(x) => Self::Full(x),
                    TrySendError::Closed(x) => Self::Closed(x),
                }
            }
        }
    }

    pub fn spawn_stream_from_iterator<T: Send + 'static>(
        it: impl Iterator<Item = T> + Send + 'static,
    ) -> (impl Stream<Item = T> + 'static) {
        let (mut s, r) = mpsc::channel(1);
        spawn(async move {
            for x in it {
                s.send(x).await?;
            }
            Ok::<(), mpsc::error::SendError<T>>(())
        });
        r
    }

    pub fn spawn_stream_from_iterator_yield<T: Send + 'static>(
        it: impl Iterator<Item = Option<T>> + Send + 'static,
    ) -> (impl Stream<Item = T> + 'static) {
        let (mut s, r) = mpsc::channel(1);
        spawn(async move {
            for x in it {
                match x {
                    Some(y) => s.send(y).await?,
                    None => poll_fn(|cx| s.poll_ready(cx)).await?, // Make sure the receiver is still open
                }
            }
            Ok::<(), error::MpscSendError<T>>(())
        });
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::stream::StreamExt;

    #[tokio::test]
    async fn test_primes_unbounded() {
        let mut primes = primes_unbounded();
        assert_eq!(primes.next().await, Some(2));
        assert_eq!(primes.next().await, Some(3));
        assert_eq!(primes.next().await, Some(5));
        assert_eq!(primes.next().await, Some(7));
    }
}
