use super::DurationSeconds;
use super::FileSize;
use super::MediaFile;
use super::Movie;
use crate::app::series::listing::Episode;
use crate::app::series::listing::Season;
use crate::app::series::listing::SeasonSummary;
use crate::app::series::listing::Series;
use crate::app::MediaId;
pub const TEST_VIDEO: &str = "https://www.w3schools.com/html/mov_bbb.mp4";

pub fn mock_series() -> Vec<Series> {
    vec![
        Series {
            id: MediaId(101),
            title: "Breaking Bad".into(),
            poster: "https://picsum.photos/seed/breakingbad/300/450".into(),
            description: Some("مدرس كيمياء يتحول إلى تاجر مخدرات.".into()),
            season_count: 5,
            season_summaries: vec![
                SeasonSummary {
                    season_number: 1,
                    episode_count: 3,
                },
                SeasonSummary {
                    season_number: 2,
                    episode_count: 2,
                },
            ],
        },
        Series {
            id: MediaId(102),
            title: "Stranger Things".into(),
            poster: "https://picsum.photos/seed/strangerthings/300/450".into(),
            description: Some("مجموعة من الأطفال يكشفون أسرارًا خارقة في بلدتهم.".into()),
            season_count: 4,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 2,
            }],
        },
        Series {
            id: MediaId(103),
            title: "The Crown".into(),
            poster: "https://picsum.photos/seed/thecrown/300/450".into(),
            description: Some("عهد الملكة إليزابيث الثانية.".into()),
            season_count: 4,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 1,
            }],
        },
        Series {
            id: MediaId(104),
            title: "Game of Thrones".into(),
            poster: "https://picsum.photos/seed/got/300/450".into(),
            description: Some("عائلات نبيلة تتصارع على السيطرة على ويستروس.".into()),
            season_count: 8,
            season_summaries: vec![SeasonSummary {
                season_number: 1,
                episode_count: 2,
            }],
        },
    ]
}

pub fn mock_season(series_id: i64, season_number: u32) -> Option<Season> {
    let episodes = match (series_id, season_number) {
        (101, 1) => vec![ep(1011, 1, 1), ep(1012, 1, 2), ep(1013, 1, 3)],
        (101, 2) => vec![ep(1014, 2, 1), ep(1015, 2, 2)],
        (102, 1) => vec![ep(1021, 1, 1), ep(1022, 1, 2)],
        (103, 1) => vec![ep(1031, 1, 1)],
        (104, 1) => vec![ep(1041, 1, 1), ep(1042, 1, 2)],
        _ => return None,
    };
    Some(Season {
        season_number,
        episodes,
    })
}

pub fn ep(id: i64, season: u32, episode: u32) -> Episode {
    Episode {
        id,
        season,
        episode,
        file: fake_media_file(),
        duration: fake_duration(3600), // 1 hour per episode
    }
}

pub(crate) fn fake_media_file() -> MediaFile {
    MediaFile {
        path: TEST_VIDEO.into(),
        size: FileSize(2_100_000_000), // ~2.1 GB
    }
}

pub(crate) fn fake_duration(seconds: u64) -> DurationSeconds {
    DurationSeconds(seconds)
}

pub(crate) fn mock_movies() -> Vec<Movie> {
    vec![
        Movie {
            id: MediaId(1),
            title: "Inception".into(),
            poster: "https://picsum.photos/seed/inception/300/450".into(),
            description: Some("لص يسرق أسرار الشركات من خلال تقنية مشاركة الأحلام.".into()),
            file: fake_media_file(),
            duration: fake_duration(8880), // 2h28m
        },
        Movie {
            id: MediaId(2),
            title: "The Matrix".into(),
            poster: "https://picsum.photos/seed/matrix/300/450".into(),
            description: Some("هاكر كمبيوتر يكتشف حقيقة الواقع.".into()),
            file: fake_media_file(),
            duration: fake_duration(8160),
        },
        Movie {
            id: MediaId(3),
            title: "Interstellar".into(),
            poster: "https://picsum.photos/seed/interstellar/300/450".into(),
            description: Some("فريق من المستكشفين يسافرون عبر ثقب دودي في الفضاء.".into()),
            file: fake_media_file(),
            duration: fake_duration(10140),
        },
        Movie {
            id: MediaId(4),
            title: "The Dark Knight".into(),
            poster: "https://picsum.photos/seed/darkknight/300/450".into(),
            description: Some("عندما يهدد الجوكر مدينة غوثام بالدمار.".into()),
            file: fake_media_file(),
            duration: fake_duration(9120),
        },
        Movie {
            id: MediaId(5),
            title: "Pulp Fiction".into(),
            poster: "https://picsum.photos/seed/pulpfiction/300/450".into(),
            description: Some("تتشابك حياة اثنين من القتلة وملاكم وزوجين من اللصوص.".into()),
            file: fake_media_file(),
            duration: fake_duration(9240),
        },
    ]
}
