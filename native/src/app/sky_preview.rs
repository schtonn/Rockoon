use super::*;
use crate::sky::Frame;
impl App {
    pub(super) fn render_sky(&mut self, full: bool) -> Task<Message> {
        if self.skybox.is_none() {
            return Task::none();
        }
        let frame = Frame {
            epoch: self.preview_revision,
            yaw: self.sky_yaw,
            pitch: self.sky_pitch,
            full,
        };
        match self.sky_frames.request(frame) {
            Some(frame) => self.sky_task(frame),
            None => Task::none(),
        }
    }
    fn sky_task(&self, frame: Frame) -> Task<Message> {
        let Some(skybox) = self.skybox.clone() else {
            return Task::none();
        };
        Task::perform(
            background(move || Ok(skybox.render(frame.yaw, frame.pitch, frame.full))),
            move |image| Message::SkyRendered(frame, image),
        )
    }
    pub(super) fn sky_rendered(
        &mut self,
        frame: Frame,
        result: Result<iced::widget::image::Handle, String>,
    ) -> Task<Message> {
        if self.sky_frames.active != Some(frame) || frame.epoch != self.preview_revision {
            return Task::none();
        }
        match result {
            Ok(image) => self.preview = Some(image),
            Err(error) => self.error(error),
        }
        match self.sky_frames.complete(frame) {
            Some(frame) => self.sky_task(frame),
            None => Task::none(),
        }
    }
}
