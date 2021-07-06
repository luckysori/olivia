use futures::channel::oneshot::Sender;
use olivia_core::{Event, EventId, PathRef, PrefixPath};
pub mod redis;
pub mod subscriber;
pub mod time_ticker;

#[cfg(test)]
mod time_tests;

pub struct Update<E> {
    pub update: E, // An Event or EventOutcome
    pub processed_notifier: Option<Sender<bool>>,
}

impl<E> From<E> for Update<E> {
    fn from(update: E) -> Self {
        Self {
            update,
            processed_notifier: None,
        }
    }
}

impl From<EventId> for Update<Event> {
    fn from(id: EventId) -> Self {
        Update::from(Event::from(id))
    }
}

impl<E: PrefixPath> PrefixPath for Update<E> {
    fn prefix_path(mut self, path: PathRef<'_>) -> Self {
        self.update = self.update.prefix_path(path);
        self
    }

    fn strip_prefix_path(mut self, path: PathRef<'_>) -> Self {
        self.update = self.update.strip_prefix_path(path);
        self
    }
}

pub type Stream<T> = std::pin::Pin<Box<dyn futures::Stream<Item = Update<T>> + Send>>;
