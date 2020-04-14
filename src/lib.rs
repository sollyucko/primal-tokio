#![feature(async_closure)]
#![feature(type_ascription)]

use primal::Primes;
use tokio::stream::Stream;

pub fn primes_unbounded() -> impl Stream<Item = usize> {
    utils::spawn_stream_from_iterator(Primes::all())
}

mod utils {
    use tokio::stream::Stream;
    use tokio::sync::mpsc;
    use tokio::task::spawn;

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
