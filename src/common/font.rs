use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::sync::Arc;
use swash::text::Codepoint;
use swash::{FontRef, StringId, Tag};

#[cfg(feature = "woff2")]
use woff2::convert_woff2_to_ttf;

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
    name: String,
    re: FontRef<'a>,
    font_type: FontType,
    supported_scripts: HashSet<CScript>,
    packed_font_data: Option<Vec<u8>>,
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

    pub fn from_data(data: &'a mut Vec<u8>) -> Self {
        assert!(data.len() >= 4);
        let (font_type, re, packed_data) = if &data[0..4] == b"\x00\x01\x00\x00" {
            (FontType::TTF, FontRef::from_index(data, 0).unwrap(), None)
        } else if &data[0..4] == b"OTTO" {
            (FontType::OTF, FontRef::from_index(data, 0).unwrap(), None)
        } else if &data[0..4] == b"wOF2" {
            #[cfg(feature = "woff2")]
            {
                let cv = convert_woff2_to_ttf(&mut data.as_slice()).unwrap();
                let pack = data.clone();

                data.clear();
                data.extend_from_slice(cv.as_slice());

                (
                    FontType::WOFF2,
                    FontRef::from_index(data, 0).unwrap(),
                    Some(pack),
                )
            }
            #[cfg(not(feature = "woff2"))]
            unimplemented!("activate the woff2 feature for this font")
        } else if &data[0..4] == b"wOFF" {
            unimplemented!("woff1 is currently not supported");
        } else {
            unimplemented!("unrecognized font magic {:?}", &data[0..4]);
        };

        let font_name = match re
            .localized_strings()
            .find(|s| s.id() == StringId::PostScript)
        {
            None => "FontNameNotFound".to_string(),
            Some(locale) => locale.to_string(),
        };

        Font {
            name: font_name,
            re,
            font_type,
            supported_scripts: Font::identify_scripts_in_font(&re),
            packed_font_data: packed_data,
        }
    }

    pub(crate) fn reference(&self) -> &FontRef<'a> {
        &self.re
    }

    pub(crate) fn font_type(&self) -> &FontType {
        &self.font_type
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn packed(&self) -> &Option<Vec<u8>> {
        &self.packed_font_data
    }

    #[allow(dead_code)]
    pub(crate) fn supported_features(&self) -> impl IntoIterator<Item = (Tag, u16)> + '_ {
        self.reference().features().map(|f| (f.tag(), 1))
    }
}

impl<'a> PartialEq<Self> for Font<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl<'a> Eq for Font<'a> {}

impl<'a> Hash for Font<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
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
}

#[derive(Default)]
pub struct FontSetBuilder<'a> {
    fonts: Vec<Font<'a>>,
}

impl<'a> FontSetBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, font: Font<'a>) -> Self {
        if self.fonts.iter().any(|x| x.name == font.name) {
            eprintln!(
                "Skipped duplicate font / second font with duplicate name: {}",
                font.name
            )
        } else {
            self.fonts.push(font);
        }
        self
    }

    pub fn extend(mut self, fonts: Vec<Font<'a>>) -> Self {
        for font in fonts {
            self = self.push(font);
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
    #[allow(dead_code)]
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
            u: unicode_script::Script::Unknown,
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
                s: Codepoint::script(cr),
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
                s: Codepoint::script(cr),
            }
        }
    }
}
