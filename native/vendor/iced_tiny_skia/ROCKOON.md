# Local renderer patch

Source: iced_tiny_skia 0.14.1, https://github.com/iced-rs/iced (MIT, see LICENSE).

Windows `window/compositor.rs::present` redraws the entire surface for every
requested frame. This bypasses incremental damage/history tracking, which can
miss old text extents and leave incomplete or stale pixels when paragraphs are
replaced. No new frame timer is introduced. Other platforms retain the upstream
path. The tradeoff is more software drawing per frame, especially on the animated
Ballance page. The user still needs to verify the reported artifacts on their
machine; this is not a claim that every source of UI latency has been resolved.

Everything else is unchanged from the published crate.
