use leptos_router::{AsPath, ParamSegment, StaticSegment};

// ─── Static segments ──────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Static {
    /// The empty segment — used for `/`.
    Root,
    /// `login`
    Login,
    /// `s` — the section route prefix.
    Section,
    /// `item` — the item route prefix.
    Item,
}

impl AsPath for Static {
    fn as_path(&self) -> &'static str {
        match self {
            Self::Root => "",
            Self::Login => "login",
            Self::Section => "s",
            Self::Item => "item",
        }
    }
}

// ─── Parameter segments ───────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Param {
    Slug,
    Id,
    ItemId,
}

impl AsPath for Param {
    fn as_path(&self) -> &'static str {
        match self {
            Self::Slug => "slug",
            Self::Id => "id",
            Self::ItemId => "item_id",
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────

const fn s(seg: Static) -> StaticSegment<Static> {
    StaticSegment(seg)
}

fn p(seg: Param) -> ParamSegment {
    ParamSegment(seg.as_path())
}

// ─── Route tables ─────────────────────────────────────────────────────
//
// These mirror exactly what the `path!` macro would have produced.
//
//   path!("/login")                       → (s(Static::Login),)
//   path!("")                             → ()
//   path!("/")                            → (s(Static::Root),)
//   path!("/s/:slug")                     → (s(Static::Section), p(Param::Slug))
//   path!("/s/:slug/:id")                 → (s(Static::Section), p(Param::Slug), p(Param::Id))
//   path!("/s/:slug/:id/item/:item_id")   → (s(Static::Section), p(Param::Slug),
//                                            p(Param::Id), s(Static::Item), p(Param::ItemId))

pub const LOGIN: (StaticSegment<Static>,) = (s(Static::Login),);

pub const PARENT: () = ();

pub const ROOT: (StaticSegment<Static>,) = (s(Static::Root),);

pub fn section() -> (StaticSegment<Static>, ParamSegment) {
    (s(Static::Section), p(Param::Slug))
}

pub fn collection() -> (StaticSegment<Static>, ParamSegment, ParamSegment) {
    (s(Static::Section), p(Param::Slug), p(Param::Id))
}

pub fn item() -> (
    StaticSegment<Static>,
    ParamSegment,
    ParamSegment,
    StaticSegment<Static>,
    ParamSegment,
) {
    (
        s(Static::Section),
        p(Param::Slug),
        p(Param::Id),
        s(Static::Item),
        p(Param::ItemId),
    )
}
