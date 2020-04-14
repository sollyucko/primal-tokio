#![feature(async_closure)]
#![feature(type_ascription)]

use primal::Primes;
use tokio::stream::{Stream, StreamExt};

pub fn primes_unbounded() -> impl Stream<Item = usize> {
    utils::spawn_stream_from_iterator(Primes::all())
}

mod utils {
    use tokio::stream::Stream;
    use tokio::sync::mpsc;
    use tokio::task::spawn_blocking;

    pub fn spawn_stream_from_iterator<T: Send + 'static>(
        it: impl Iterator<Item = T> + Send + 'static,
    ) -> impl Stream<Item = T> + 'static {
        let (mut s, r) = mpsc::channel(1);
        spawn_blocking(async move || -> Result<(), mpsc::error::SendError<T>> {
            for x in it {
                s.send(x).await?;
            }
            Ok(())
        });
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tokio::io::{self, stdout, AsyncWriteExt};
    use tokio::join;
    use tokio::task::{spawn, yield_now};
    use tokio::time::{delay_for, Duration};

    #[tokio::test]
    async fn test_primes_unbounded() {
        let mut primes = primes_unbounded();
        assert_eq!(primes.next().await, Some(2));
        assert_eq!(primes.next().await, Some(3));
        assert_eq!(primes.next().await, Some(5));
        assert_eq!(primes.next().await, Some(7));
    }
}
