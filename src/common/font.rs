use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::sync::Arc;
use swash::text::Codepoint;
use swash::{FontRef, Tag};
use unicode_script::{Script};

#[derive(Clone)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub(crate) enum FontType {
    OTF,
    TTF,
    WOFF,
    WOFF2,
}

impl FontType {
    pub(crate) fn embed_tag(&self) -> &'static str {
        match self {
            FontType::OTF => "application/font-otf",
            FontType::TTF => "application/font-ttf",
            FontType::WOFF => "application/font-woff",
            FontType::WOFF2 => "application/font-woff2",
        }
    }
}

pub struct Font<'a> {
    font_name: String,
    re: FontRef<'a>,
    typ: FontType,
    supported_scripts: HashSet<CScript>,
}

impl<'a> Font<'a> {
    fn identify_scripts_in_font(fr: &FontRef) -> HashSet<CScript> {
        let mut scripts_in_specs = fr
            .writing_systems()
            .filter_map(|s| s.script())
            .map(CScript::try_from)
            .filter_map(|s| s.ok())
            .collect::<HashSet<CScript>>();

        if scripts_in_specs.is_empty() {
            fr.charmap().enumerate(|i, _g| {
                if let Ok(c) = <u32 as TryInto<char>>::try_into(i) {
                    match CScript::try_from(c.script()) {
                        Ok(cs) => {
                            scripts_in_specs.insert(cs);
                        }
                        Err(_e) => {}
                    };
                }
            });
        }

        scripts_in_specs
    }

    pub fn from_data(name: String, data: &'a [u8]) -> Self {
        let re = FontRef::from_index(data, 0).unwrap();

        Font {
            font_name: name,
            re,
            typ: FontType::TTF,
            supported_scripts: Font::identify_scripts_in_font(&re),
        }
    }

    pub(crate) fn reference(&self) -> &FontRef<'a> {
        &self.re
    }

    pub(crate) fn font_type(&self) -> &FontType {
        &self.typ
    }

    pub(crate) fn name(&self) -> &str {
        &self.font_name
    }

    pub(crate) fn supported_features(&self) -> impl IntoIterator<Item = (Tag, u16)> + '_ {
        self.reference().features().map(|f| (f.tag(), 1))
    }
}

impl<'a> PartialEq<Self> for Font<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.font_name.eq(&other.font_name)
    }
}

impl<'a> Eq for Font<'a> {}

impl<'a> Hash for Font<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_name.hash(state)
    }
}

#[derive(Clone)]
pub struct FontSet<'a> {
    inner: Arc<Vec<Font<'a>>>,
}

impl<'a> FontSet<'a> {
    pub(crate) fn get_font_for_script(&self, script: &CScript) -> Option<&Font> {
        self.inner
            .iter()
            .find(|f| f.supported_scripts.contains(script))
    }

    pub(crate) fn get_fonts(&self) -> &Arc<Vec<Font<'a>>> {
        &self.inner
    }
}

pub struct FontSetBuilder<'a> {
    fonts: Vec<Font<'a>>,
}

impl<'a> FontSetBuilder<'a> {
    pub fn new() -> Self {
        FontSetBuilder {
            fonts: Vec::default(),
        }
    }
    pub fn add_font(mut self, font: Font<'a>) -> Self {
        self.fonts.push(font);
        self
    }

    pub fn add_fonts(mut self, fonts: Vec<Font<'a>>) -> Self {
        for font in fonts {
            self = self.add_font(font);
        }
        self
    }

    pub fn build(self) -> FontSet<'a> {
        FontSet {
            inner: Arc::new(self.fonts),
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct CScript {
    u: unicode_script::Script,
    s: swash::text::Script,
}

impl CScript {
    pub fn u(&self) -> unicode_script::Script {
        self.u
    }

    pub fn s(&self) -> swash::text::Script {
        self.s
    }
}

impl TryFrom<swash::text::Script> for CScript {
    type Error = ();

    fn try_from(value: swash::text::Script) -> Result<CScript, ()> {
        match unicode_script::Script::from_full_name(value.name().replace(' ', "_").as_str()) {
            None => Err(()),
            Some(u) => Ok(CScript { u, s: value }),
        }
    }
}

impl Default for CScript {
    fn default() -> Self {
        CScript {
            u: Script::Unknown,
            s: swash::text::Script::Unknown,
        }
    }
}

pub trait GuessScript {
    fn guess_script(&self) -> CScript;
}

impl GuessScript for String {
    fn guess_script(&self) -> CScript {
        if self.is_empty() {
            CScript::default()
        } else {
            let cr = self.chars().next().unwrap();
            CScript {
                u: unicode_script::UnicodeScript::script(&cr),
                s: swash::text::Codepoint::script(cr),
            }
        }
    }
}

impl GuessScript for &str {
    fn guess_script(&self) -> CScript {
        if self.is_empty() {
            CScript::default()
        } else {
            let cr = self.chars().next().unwrap();
            CScript {
                u: unicode_script::UnicodeScript::script(&cr),
                s: swash::text::Codepoint::script(cr),
            }
        }
    }
}
