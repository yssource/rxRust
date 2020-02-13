use crate::prelude::*;
use crate::subject::{LocalSubject, SharedSubject};
use ops::ref_count::{LocalRefCount, SharedRefCount};

#[derive(Clone)]
pub struct ConnectableObservable<Source, Subject> {
  pub(crate) source: Source,
  pub(crate) subject: Subject,
}

pub type LocalConnectableObservable<'a, S, Item, Err> =
  ConnectableObservable<S, LocalSubject<'a, Item, Err>>;

pub type SharedConnectableObservable<S, Item, Err> =
  ConnectableObservable<S, SharedSubject<Item, Err>>;

macro observable_impl($subscription:ty, $($marker:ident +)* $lf: lifetime) {
  type Unsub = $subscription;
  #[inline(always)]
  fn actual_subscribe<O: Observer<Self::Item, Self::Err> + $($marker +)* $lf>(
    self,
    subscriber: Subscriber<O, $subscription>,
  ) -> Self::Unsub {
    self.subject.actual_subscribe(subscriber)
  }
}

impl<'a, S, Item, Err> Observable<'a>
  for LocalConnectableObservable<'a, S, Item, Err>
where
  S: Observable<'a, Item = Item, Err = Err>,
  S: Observable<'a, Item = Item, Err = Err>,
{
  type Item = Item;
  type Err = Err;
  observable_impl!(LocalSubscription, 'a);
}

impl<S, Item, Err> SharedObservable
  for SharedConnectableObservable<S, Item, Err>
where
  S: SharedObservable<Item = Item, Err = Err>,
  S: SharedObservable<Item = Item, Err = Err>,
{
  type Item = Item;
  type Err = Err;
  observable_impl!(SharedSubscription, Send + Sync + 'static);
}

impl<'a, Item, Err, S> LocalConnectableObservable<'a, S, Item, Err>
where
  S: Observable<'a, Item = Item, Err = Err>,
{
  pub fn local(observable: S) -> Self {
    Self {
      source: observable,
      subject: Subject::local(),
    }
  }

  #[inline]
  pub fn ref_count(self) -> LocalRefCount<'a, S, Item, Err> {
    LocalRefCount::new(self)
  }
}

impl<S, Item, Err> ConnectableObservable<S, SharedSubject<Item, Err>>
where
  S: SharedObservable<Item = Item, Err = Err>,
{
  pub fn shared(observable: S) -> Self {
    ConnectableObservable {
      source: observable,
      subject: Subject::shared(),
    }
  }
  #[inline]
  pub fn ref_count(self) -> SharedRefCount<S, Item, Err> {
    SharedRefCount::new(self)
  }
}

impl<'a, S, Item, Err> LocalConnectableObservable<'a, S, Item, Err>
where
  S: Observable<'a, Item = Item, Err = Err>,
  Item: Copy + 'a,
  Err: Copy + 'a,
{
  pub fn connect(self) -> S::Unsub {
    self.source.actual_subscribe(Subscriber {
      observer: self.subject.observers,
      subscription: self.subject.subscription,
    })
  }
}

impl<S, Item, Err> SharedConnectableObservable<S, Item, Err>
where
  S: SharedObservable<Item = Item, Err = Err>,
  Item: Copy + Send + Sync + 'static,
  Err: Copy + Send + Sync + 'static,
{
  pub fn connect(self) -> S::Unsub {
    self.source.actual_subscribe(Subscriber {
      observer: self.subject.observers,
      subscription: self.subject.subscription,
    })
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn smoke() {
    let o = observable::of(100);
    let connected = ConnectableObservable::local(o);
    let mut first = 0;
    let mut second = 0;
    let _guard1 = connected.clone().subscribe(|v| first = v);
    let _guard2 = connected.clone().subscribe(|v| second = v);

    connected.connect();
    assert_eq!(first, 100);
    assert_eq!(second, 100);
  }

  #[test]
  fn fork_and_shared() {
    let o = observable::of(100);
    let connected = ConnectableObservable::shared(o);
    connected.clone().to_shared().subscribe(|_| {});
    connected.clone().to_shared().subscribe(|_| {});

    connected.connect();
  }
}
