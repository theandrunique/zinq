use std::{collections::{HashMap, HashSet}, sync::LazyLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultEmoji {
    pub id: &'static str,
    pub shortcode: Option<&'static str>,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultPack {
    pub pack_id: i64,
    pub name: &'static str,
    pub emojis: Vec<DefaultEmoji>,
}

pub struct RawPackDef {
    pub id: i64,
    pub name: &'static str,
    pub raw_emojis: &'static [(&'static str, Option<&'static str>)],
}

pub const DEFAULT_PACKS_RAW: &[RawPackDef] = &[
    RawPackDef {
        id: 0,
        name: "Reactions",
        raw_emojis: &[
            ("thumbs_up", Some(":thumbs_up:")),
            ("thumbs_down", Some(":thumbs_down:")),
            ("heart", Some(":heart:")),
            ("fire", Some(":fire:")),
            ("tada", Some(":tada:")),
            ("star_struck", Some(":star_struck:")),
            ("scream", Some(":scream:")),
            ("grin", Some(":grin:")),
            ("cry", Some(":cry:")),
            ("poop", Some(":poop:")),
            ("vomit", Some(":vomiting_face:")),
            ("hearts", Some(":smiling_face_with_hearts:")),
            ("exploding_head", Some(":exploding_head:")),
            ("thinking", Some(":thinking:")),
            ("cursing", Some(":face_with_symbols_on_mouth:")),
            ("clap", Some(":clap:")),
        ],
    },
];

pub static DEFAULT_PACKS: LazyLock<Vec<DefaultPack>> = LazyLock::new(|| {
    let mut seen_pack_ids = HashSet::new();
    for def in DEFAULT_PACKS_RAW {
        assert!(seen_pack_ids.insert(def.id), "Duplicate pack_id detected: {}", def.id);
    }

    DEFAULT_PACKS_RAW.iter().map(|def| {
        let emojis = def.raw_emojis
            .iter()
            .enumerate()
            .map(|(i, &(id, shortcode))| DefaultEmoji {
                id,
                shortcode,
                order: i as i32,
            })
            .collect();

        DefaultPack {
            pack_id: def.id,
            name: def.name,
            emojis,
        }
    }).collect()
});

pub static DEFAULT_EMOJI_MAP: LazyLock<HashMap<&'static str, DefaultEmoji>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for pack in DEFAULT_PACKS.iter() {
        for emoji in &pack.emojis {
            assert!(map.insert(emoji.id, emoji.clone()).is_none(), "Duplicate emoji id: {}", emoji.id);
        }
    }
    map
});

#[derive(Debug, Serialize)]
pub struct EmojiManifest {
    pub version: u32,
    pub packs: Vec<DefaultPack>,
}

pub fn build_default_manifest() -> EmojiManifest {
    EmojiManifest {
        version: 1,
        packs: DEFAULT_PACKS.clone(),
    }
}
