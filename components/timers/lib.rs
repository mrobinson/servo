/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic timer scheduler module that can be used to launch a per-process
//! background thread and schedule timers to fire from that thread.

#![deny(unsafe_code)]

use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use base::id::PipelineId;
use crossbeam_channel::{Receiver, RecvError, RecvTimeoutError, Sender};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

static TIMER_SCHEDULER_PROXY: OnceLock<Arc<TimerSchedulerProxy>> = OnceLock::new();

/// Describes the source that requested the TimerEvent.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum TimerSource {
    /// The event was requested from a window (ScriptThread).
    FromWindow(PipelineId),
    /// The event was requested from a worker (DedicatedGlobalWorkerScope).
    FromWorker,
}

/// The id to be used for a `TimerEvent` is defined by the corresponding `TimerEventRequest`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct TimerEventId(pub u32);

/// Notifies the script thread to fire due timers.
/// `TimerSource` must be `FromWindow` when dispatched to `ScriptThread` and
/// must be `FromWorker` when dispatched to a `DedicatedGlobalWorkerScope`
#[derive(Debug, Deserialize, Serialize)]
pub struct TimerEvent(pub TimerSource, pub TimerEventId);

/// A callback to pass to the [`TimerScheduler`] to be called when the timer is
/// dispatched.
pub type BoxedTimerCallback = Box<dyn Fn(TimerEvent) + Send + 'static>;

/// Requests a TimerEvent-Message be sent after the given duration.
pub struct TimerEventRequest {
    pub callback: BoxedTimerCallback,
    pub source: TimerSource,
    pub id: TimerEventId,
    pub duration: Duration,
}

impl TimerEventRequest {
    fn dispatch(self) {
        (self.callback)(TimerEvent(self.source, self.id))
    }
}

struct ScheduledEvent {
    request: TimerEventRequest,
    for_time: Instant,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &ScheduledEvent) -> cmp::Ordering {
        self.for_time.cmp(&other.for_time).reverse()
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &ScheduledEvent) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ScheduledEvent {}
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &ScheduledEvent) -> bool {
        std::ptr::eq(self, other)
    }
}

/// The message used to send requests to the [`TimerScheduler`] thread.
enum TimerSchedulerMessage {
    Request(TimerEventRequest),
    Timeout,
}

/// The internal [`TimerScheduler`] which handles incoming requests from the
/// [`TimerSchedulerProxy`] in a separate thread.
pub struct TimerScheduler {
    /// A priority queue of future events, sorted by due time.
    queue: BinaryHeap<ScheduledEvent>,
    /// A receiver to receive requests from other threads.
    receiver: Receiver<TimerSchedulerMessage>,
}

impl TimerScheduler {
    /// If there are queued events, wait for a new message or for when the next timer should
    /// fire. If there are no queued events, just wait for the next event.
    fn wait_for_event(&self, now: Instant) -> Result<TimerSchedulerMessage, RecvError> {
        self.queue
            .peek()
            .map(
                |event| match self.receiver.recv_timeout(event.for_time - now) {
                    Ok(message) => Ok(message),
                    Err(timeout_error) => match timeout_error {
                        RecvTimeoutError::Timeout => Ok(TimerSchedulerMessage::Timeout),
                        RecvTimeoutError::Disconnected => Err(RecvError),
                    },
                },
            )
            .unwrap_or_else(|| self.receiver.recv())
    }

    /// Dispatch any timer events from this [`TimerScheduler`]'s `queue` when `now` is
    /// past the due time of the event.
    fn dispatch_completed_timers(&mut self, now: Instant) {
        loop {
            match self.queue.peek() {
                // Dispatch the event if its due time is past.
                Some(event) if event.for_time <= now => {},
                // Otherwise, we're done dispatching events.
                _ => break,
            }
            // Remove the event from the priority queue (Note this only executes when the
            // first event has been dispatched
            self.queue
                .pop()
                .expect("Exepcted request")
                .request
                .dispatch();
        }
    }

    fn schedule_timer(&mut self, request: TimerEventRequest, now: Instant) {
        let for_time = now + request.duration;
        self.queue.push(ScheduledEvent { request, for_time });
    }

    pub fn start() -> TimerSchedulerProxy {
        let (sender, receiver) = crossbeam_channel::unbounded();

        thread::Builder::new()
            .name(String::from("TimerScheduler"))
            .spawn(move || {
                let mut timer_scheduler = TimerScheduler {
                    queue: Default::default(),
                    receiver,
                };

                let mut now = Instant::now();
                loop {
                    let message = timer_scheduler.wait_for_event(now);
                    now = Instant::now();

                    match message {
                        Ok(TimerSchedulerMessage::Request(request)) => {
                            timer_scheduler.schedule_timer(request, now)
                        },
                        Ok(TimerSchedulerMessage::Timeout) => {},
                        Err(error) => warn!("TimerScheduler had a receiver error: {error:?}"),
                    }

                    // Always dispatch any completed timers. This is done after handling messages, so that timers
                    // that are scheduled after 'now' are immediately dispatched.
                    timer_scheduler.dispatch_completed_timers(now);
                }
            })
            .expect("TimerScheduler thread creation failed.");

        TimerSchedulerProxy { sender }
    }
}

/// A proxy for the [`TimerScheduler`] which only exists on its own therad.
pub struct TimerSchedulerProxy {
    sender: Sender<TimerSchedulerMessage>,
}
impl TimerSchedulerProxy {
    /// Get the process global [`TimerSchedulerProxy`].
    pub fn get() -> Arc<TimerSchedulerProxy> {
        TIMER_SCHEDULER_PROXY
            .get_or_init(|| Arc::new(TimerScheduler::start()))
            .clone()
    }

    /// Handle an incoming timer request.
    pub fn schedule_timer(&self, request: TimerEventRequest) {
        let _ = self.sender.send(TimerSchedulerMessage::Request(request));
    }
}
