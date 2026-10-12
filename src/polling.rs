//! Long polling as a `Stream` of updates.

use std::collections::VecDeque;
use std::time::Duration;

use futures_util::Stream;

use crate::Bot;
use crate::error::Result;
use crate::types::Update;

/// How [`Bot::updates`] polls.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PollOptions {
    /// Long-poll timeout per request.
    pub timeout: Duration,
    /// Call `deleteWebhook` when a webhook blocks polling (error 409) instead of failing.
    pub delete_webhook: bool,
    /// Longest wait between retries after errors.
    pub max_backoff: Duration,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            delete_webhook: false,
            max_backoff: Duration::from_secs(60),
        }
    }
}

impl PollOptions {
    /// Sets the long-poll timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Removes a webhook that blocks polling.
    pub fn delete_webhook(mut self, delete: bool) -> Self {
        self.delete_webhook = delete;
        self
    }

    /// Caps the retry backoff.
    pub fn max_backoff(mut self, max: Duration) -> Self {
        self.max_backoff = max;
        self
    }
}

struct State {
    bot: Bot,
    options: PollOptions,
    queue: VecDeque<Update>,
    failures: u32,
    done: bool,
}

impl Bot {
    /// Polls `getUpdates` forever and yields each update.
    ///
    /// Transient errors (network, timeouts, quota, 5xx) are yielded as `Err` and polling continues
    /// after an exponential backoff, so you can log them. Other errors (an invalid token, a
    /// webhook still set) are yielded once and end the stream. Drop the stream to stop.
    ///
    /// ```no_run
    /// # async fn run(bot: zalo_bot::Bot) {
    /// use futures_util::StreamExt;
    ///
    /// let mut updates = std::pin::pin!(bot.updates(Default::default()));
    /// while let Some(update) = updates.next().await {
    ///     match update {
    ///         Ok(update) => println!("{:?}", update.content()),
    ///         Err(error) => eprintln!("{error}"),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn updates(&self, options: PollOptions) -> impl Stream<Item = Result<Update>> + Send {
        let state = State {
            bot: self.clone(),
            options,
            queue: VecDeque::new(),
            failures: 0,
            done: false,
        };
        futures_util::stream::unfold(state, |mut s| async move {
            loop {
                if let Some(update) = s.queue.pop_front() {
                    return Some((Ok(update), s));
                }
                if s.done {
                    return None;
                }
                if s.failures > 0 {
                    let wait =
                        Duration::from_secs(1u64 << s.failures.min(6)).min(s.options.max_backoff);
                    tokio::time::sleep(wait).await;
                }
                match s.bot.get_updates(s.options.timeout).await {
                    Ok(updates) => {
                        s.failures = 0;
                        s.queue.extend(updates);
                    }
                    Err(e) if e.is_webhook_conflict() && s.options.delete_webhook => {
                        if let Err(error) = s.bot.delete_webhook().await {
                            s.done = true;
                            return Some((Err(error), s));
                        }
                    }
                    Err(e) if e.is_retryable() => {
                        s.failures = s.failures.saturating_add(1);
                        return Some((Err(e), s));
                    }
                    Err(e) => {
                        s.done = true;
                        return Some((Err(e), s));
                    }
                }
            }
        })
    }
}
