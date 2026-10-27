use iced::{
    Rectangle, Task,
    advanced::widget::{Id, Operation, operation::Outcome},
};
struct Query {
    found: Option<Rectangle>,
}
impl Operation<Option<Rectangle>> for Query {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<Rectangle>>)) {
        operate(self);
    }
    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&Id::new("tas-preview")) {
            self.found = Some(bounds);
        }
    }
    fn finish(&self) -> Outcome<Option<Rectangle>> {
        Outcome::Some(self.found)
    }
}
pub fn viewport() -> Task<Option<Rectangle>> {
    iced::advanced::widget::operate(Query { found: None })
}
