use parking_lot::Mutex;
use std::sync::Arc;
use swash::scale::ScaleContext;
use swash::shape::ShapeContext;
use swash::FontRef;

#[derive(Clone)]
pub(crate) enum FontType {
    OTF,
    TTF,
    WOFF,
    WOFF2,
}

impl FontType {
    pub(crate) fn to_embed_tag(&self) -> &'static str {
        match self {
            FontType::OTF => "application/font-otf",
            FontType::TTF => "application/font-ttf",
            FontType::WOFF => "application/font-woff",
            FontType::WOFF2 => "application/font-woff2",
        }
    }
}

#[derive(Clone)]
pub(crate) struct Font<'a> {
    re: Arc<FontRef<'a>>,
    typ: FontType,
    shape: Arc<Mutex<ShapeContext>>,
    scale: Arc<Mutex<ScaleContext>>,
}

impl<'a> Font<'a> {
    pub(crate) fn new(font_ref: FontRef<'a>, font_type: FontType) -> Font<'a> {
        Font {
            re: Arc::from(font_ref),
            typ: font_type,
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

    pub fn t(&self) -> &FontType {
        &self.typ
    }
}
