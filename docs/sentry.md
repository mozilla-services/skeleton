# A note about sentry.io

Sentry is our error event logging and notification tool. It is accessed via [sentry.io](https://sentry.io).

Sentry's view is that an "event" is a special occurance that requires resolution. This has taken more solid form in later versions of their SDK and interface.
Prior, we had a history of capturing all errors into Sentry and using it as both an event notification system as well as "richer metrics", since we could apply additional information to the captured event (using tags and extra data). Those are still present, but it is becoming increasingly more difficult to do this, and there is also good indications that some critical logging data may be lost in the noise of these "richer metric" events. This leads to a fairly unsustainable approach since we would be fighting against the library and service provider rather than working with them.

To that end, we are altering guidance on how sentry is to be used to log events.

The middleware wrapper is still in place, but it may be removed sometime in the future. Developers are instead encouraged to use Sentry the way that [it is intended to be used](https://docs.rs/sentry/latest/sentry/).
