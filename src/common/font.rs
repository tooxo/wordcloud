use parking_lot::Mutex;
use std::sync::Arc;
use swash::scale::ScaleContext;
use swash::shape::ShapeContext;
use swash::FontRef;

#[derive(Clone)]
pub(crate) struct Font<'a> {
    re: Arc<FontRef<'a>>,
    shape: Arc<Mutex<ShapeContext>>,
    scale: Arc<Mutex<ScaleContext>>,
}

impl<'a> Font<'a> {
    pub(crate) fn new(font_ref: FontRef<'a>) -> Font<'a> {
        Font {
            re: Arc::from(font_ref),
            shape: Arc::from(Mutex::new(ShapeContext::new())),
            scale: Arc::from(Mutex::new(ScaleContext::new())),
        }
    }

    pub fn re(&self) -> &Arc<FontRef<'a>> {
        &self.re
    }
    pub fn shape(&self) -> &Arc<Mutex<ShapeContext>> {
        &self.shape
    }
    pub fn scale(&self) -> &Arc<Mutex<ScaleContext>> {
        &self.scale
    }
}
