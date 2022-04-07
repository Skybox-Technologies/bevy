/// Debug events for when a frame rendering starts/finishes
pub enum RenderDebugEvent {
    /// Time where frame rendering started
    Started(bevy_utils::Instant),
    /// Time where frame rendering finished
    Finished(bevy_utils::Instant),
}

#[cfg(feature = "render-time-debug")]
pub struct RenderDebugLayer {
    pub sender: crossbeam_channel::Sender<RenderDebugEvent>,
}

#[cfg(feature = "render-time-debug")]
#[derive(bevy_ecs::system::Resource)]
/// Debug channel for receiving [RenderDebugEvent]s.
pub struct RenderDebugChannel {
    /// The channel
    pub receiver: crossbeam_channel::Receiver<RenderDebugEvent>,
}

#[cfg(feature = "render-time-debug")]
impl<S> tracing_subscriber::Layer<S> for RenderDebugLayer
where
    S: bevy_utils::tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_enter(
        &self,
        id: &bevy_utils::tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if Self::check(id, ctx) {
            self.send(RenderDebugEvent::Started(bevy_utils::Instant::now()));
        }
    }

    fn on_exit(
        &self,
        id: &bevy_utils::tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if Self::check(id, ctx) {
            self.send(RenderDebugEvent::Finished(bevy_utils::Instant::now()));
        }
    }
}

#[cfg(feature = "render-time-debug")]
impl RenderDebugLayer {
    fn check<S>(
        id: &bevy_utils::tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool
    where
        S: bevy_utils::tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        ctx.metadata(id).map_or(false, |metadata| {
            metadata.name() == "frame"
                && metadata.is_span()
                && metadata.module_path() == Some("bevy_app::app")
        })
    }

    fn send(&self, event: RenderDebugEvent) {
        match self.sender.try_send(event) {
            Ok(()) => (),
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                super::warn!("Sending end of frame logging channel is disconnected")
            }
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                super::warn!("Frame logging channel is full")
            }
        }
    }
}
