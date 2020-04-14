#![feature(async_closure)]
#![feature(type_ascription)]

use primal::{Sieve, SievePrimes, Primes};
use tokio::stream::Stream;

pub fn primes_unbounded() -> impl Stream<Item = usize> {
    utils::spawn_stream_from_iterator(Primes::all())
}

struct PrimeFactor {
    n: usize,
    #[allow(dead_code)]
    sieve_box: Box<Sieve>,
    primes: SievePrimes<'static>,
}

impl Iterator for PrimeFactor {
    type Item = Option<(usize, usize)>;

    fn next(&mut self) -> Option<Option<(usize, usize)>> {
        let p = self.primes.next()?;

        if p*p > self.n {
            return Some(Some((self.n, 1)));
        }

        if self.n % p == 0 {
            let mut i = 0;
            while self.n % p == 0 {
                i += 1;
                self.n /= p;
            }
            Some(Some((p, i)))
        } else {
            Some(None)
        }
    }
}

/// Add large remainder detection
pub fn prime_factor(n: usize) -> impl Stream<Item = (usize, usize)> {
    let sieve = Sieve::new((n as f64).sqrt() as usize);
    let sieve_box = Box::new(sieve);
    let sieve_ref = unsafe { &*(&*sieve_box as *const Sieve) };
    let primes = sieve_ref.primes_from(0);
    utils::spawn_stream_from_iterator_yield(PrimeFactor { n, sieve_box, primes })
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

    #[tokio::test]
    async fn test_prime_factor() {
        let mut factors = prime_factor(98_736); // 2*2*2*2*3*11*11*17
        assert_eq!(factors.next().await, Some((2, 4)));
        assert_eq!(factors.next().await, Some((3, 1)));
        assert_eq!(factors.next().await, Some((11, 2)));
        assert_eq!(factors.next().await, Some((17, 1)));
    }
    
    #[tokio::test]
    async fn test_prime_factor_large_prime() {
        let mut factors = prime_factor(5_706_079_200_624); // 2*2*2*2*3*11*11*982_451_653
        assert_eq!(factors.next().await, Some((2, 4)));
        assert_eq!(factors.next().await, Some((3, 1)));
        assert_eq!(factors.next().await, Some((11, 2)));
        assert_eq!(factors.next().await, Some((982_451_653, 1)));
    }
}
