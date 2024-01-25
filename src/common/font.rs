use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::Cursor;

use std::sync::Arc;
use swash::text::{Codepoint};

use swash::scale::ScaleContext;
use swash::{FontRef, StringId, Tag};

#[cfg(feature = "woff2")]
use woff2::convert_woff2_to_ttf;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
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

/**
    Error returned if font loading failed
*/
pub type FontLoadingError = String;

/**
    Result from [`FontLoadingError`]
*/
pub type FontLoadingResult<T> = Result<T, FontLoadingError>;

/**
    Represents a Font stored in memory. By default, it supports `OTF` and `TTF` fonts, with
    the create features `woff` and `woff2` it also supports loading `WOFF` fonts.
*/
pub struct Font<'a> {
    name: String,
    re: FontRef<'a>,
    font_type: FontType,
    supported_scripts: HashSet<CScript>,
    packed_font_data: Option<Vec<u8>>,
    _approximate_pixel_width: f32,
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

        #[allow(clippy::unwrap_used)]
        scripts_in_specs.insert(CScript::try_from(swash::text::Script::Common).unwrap());

        scripts_in_specs
    }

    /**
        Load a font from memory. The buffer, in which the font data is stored might be changed
        after calling this function.
    */
    pub fn from_data(data: &'a mut Vec<u8>) -> FontLoadingResult<Self> {
        assert!(data.len() >= 4);
        let (font_type, re, packed_data) = if &data[0..4] == b"\x00\x01\x00\x00" {
            (FontType::TTF, FontRef::from_index(data, 0), None)
        } else if &data[0..4] == b"OTTO" {
            (FontType::OTF, FontRef::from_index(data, 0), None)
        } else if &data[0..4] == b"wOF2" {
            #[cfg(feature = "woff2")]
            {
                let cv = match convert_woff2_to_ttf(&mut data.as_slice()) {
                    Ok(c) => c,
                    Err(e) => return Err(e.to_string()),
                };
                let pack = data.clone();

                data.clear();
                data.extend_from_slice(cv.as_slice());

                (FontType::WOFF2, FontRef::from_index(data, 0), Some(pack))
            }
            #[cfg(not(feature = "woff2"))]
            unimplemented!("activate the woff2 feature for this font")
        } else if &data[0..4] == b"wOFF" {
            let mut inp_cur = Cursor::new(&data);
            let mut out_cur = Cursor::new(Vec::new());
            rs_woff::woff2otf(&mut inp_cur, &mut out_cur)
                .expect("font conversion from woff1 unsuccessful");

            let pack = data.clone();

            data.clear();
            data.extend_from_slice(out_cur.get_ref().as_slice());

            (FontType::WOFF, FontRef::from_index(data, 0), Some(pack))
        } else {
            unimplemented!("unrecognized font magic {:?}", &data[0..4]);
        };

        let re = match re {
            None => return Err(FontLoadingError::from("loading font failed")),
            Some(e) => e,
        };

        let font_name = match re
            .localized_strings()
            .find(|s| s.id() == StringId::PostScript)
        {
            None => "FontNameNotFound".to_string(),
            Some(locale) => locale.to_string(),
        };

        let mut scale_context = ScaleContext::new();
        let mut scaler = scale_context.builder(re).size(20_f32).build();
        let glyph_id = re.charmap().map('a');
        let outline = scaler.scale_outline(glyph_id).unwrap();

        Ok(Font {
            name: font_name,
            re,
            font_type,
            supported_scripts: Font::identify_scripts_in_font(&re),
            packed_font_data: packed_data,
            _approximate_pixel_width: outline.bounds().width() / 20.,
        })
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

/**
    Manages multiple [`Font`]s and selects the right one for each writing script used in the
    input text. Has to be built via the [`FontSetBuilder`].
*/
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

/**
    Builds a [`FontSet`]

    Example Use:
    ```
    use wordcloud::font::{FontSet, FontSetBuilder};

    // let fonts = ...;
    let font_set: FontSet = FontSetBuilder::new()
        .extend(fonts)
        .build();
    ```
*/
#[derive(Default)]
pub struct FontSetBuilder<'a> {
    fonts: Vec<Font<'a>>,
}

impl<'a> FontSetBuilder<'a> {
    /**
        Construct a new [`FontSetBuilder`]
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Add a new [`Font`] to the [`FontSet`]
    */
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

    /**
        Add a collection of [`Font`]s to the [`FontSet`]
    */
    pub fn extend(mut self, fonts: Vec<Font<'a>>) -> Self {
        for font in fonts {
            self = self.push(font);
        }
        self
    }

    /**
        Build a [`FontSet`] from the fonts. Panics, if no font was provided.
    */
    pub fn build(self) -> FontSet<'a> {
        if self.fonts.is_empty() {
            panic!("At least one font needs to be provided.");
        }
        FontSet {
            inner: Arc::new(self.fonts),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub(crate) struct CScript {
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

pub(crate) trait GuessScript {
    fn guess_script(&self) -> CScript;
}

impl GuessScript for String {
    fn guess_script(&self) -> CScript {
        match self.chars().next() {
            None => CScript::default(),
            Some(cr) => CScript {
                u: unicode_script::UnicodeScript::script(&cr),
                s: Codepoint::script(cr),
            },
        }
    }
}

impl GuessScript for &str {
    fn guess_script(&self) -> CScript {
        match self.chars().next() {
            None => CScript::default(),
            Some(cr) => CScript {
                u: unicode_script::UnicodeScript::script(&cr),
                s: Codepoint::script(cr),
            },
        }
    }
}
