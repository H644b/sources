	defaults::defaults_get,
	net::{HttpMethod, Request},
	String, Vec,

#![no_std]
use aidoku::{
	prelude::*,
	std::{
		net::Request,
		String, Vec,
		defaults::defaults_get,
		json::Json,
	},
	Chapter, ContentRating, Filter, FilterType, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

const BASE_URL: &str = "https://mangafire.to";

pub struct MangafireSource;

impl Source for MangafireSource {
	fn id(&self) -> String {
		"en.mangafire".into()
	}
	fn name(&self) -> String {
		"MangaFire".into()
	}
	fn lang(&self) -> String {
		"en".into()
	}

	fn filters(&self) -> Vec<Filter> {
		vec![
			Filter {
				name: "Type".into(),
				kind: FilterType::Select,
				values: vec![
					("".into(), "Any".into()),
					("manga".into(), "Manga".into()),
					("one_shot".into(), "One-Shot".into()),
					("doujinshi".into(), "Doujinshi".into()),
					("novel".into(), "Novel".into()),
					("manhwa".into(), "Manhwa".into()),
					("manhua".into(), "Manhua".into()),
				],
				id: "type".into(),
			},
			Filter {
				name: "Genre".into(),
				kind: FilterType::MultiSelect,
				values: vec![
					("1".into(), "Action".into()),
					("78".into(), "Adventure".into()),
					("3".into(), "Avant Garde".into()),
					("4".into(), "Boys Love".into()),
					("5".into(), "Comedy".into()),
					("77".into(), "Demons".into()),
					("6".into(), "Drama".into()),
					("7".into(), "Ecchi".into()),
					("79".into(), "Fantasy".into()),
					("9".into(), "Girls Love".into()),
					("10".into(), "Gourmet".into()),
					("11".into(), "Harem".into()),
					("530".into(), "Horror".into()),
					("13".into(), "Isekai".into()),
					("531".into(), "Iyashikei".into()),
					("15".into(), "Josei".into()),
					("532".into(), "Kids".into()),
					("539".into(), "Magic".into()),
					("533".into(), "Mahou Shoujo".into()),
					("534".into(), "Martial Arts".into()),
					("19".into(), "Mecha".into()),
					("535".into(), "Military".into()),
					("21".into(), "Music".into()),
					("22".into(), "Mystery".into()),
					("23".into(), "Parody".into()),
					("536".into(), "Psychological".into()),
					("25".into(), "Reverse Harem".into()),
					("26".into(), "Romance".into()),
					("73".into(), "School".into()),
					("28".into(), "Sci-Fi".into()),
					("537".into(), "Seinen".into()),
					("30".into(), "Shoujo".into()),
					("31".into(), "Shounen".into()),
					("538".into(), "Slice of Life".into()),
					("33".into(), "Space".into()),
					("34".into(), "Sports".into()),
					("75".into(), "Super Power".into()),
					("76".into(), "Supernatural".into()),
					("37".into(), "Suspense".into()),
					("38".into(), "Thriller".into()),
					("39".into(), "Vampire".into()),
				],
				id: "genre".into(),
			},
			Filter {
				name: "Status".into(),
				kind: FilterType::Select,
				values: vec![
					("".into(), "Any".into()),
					("completed".into(), "Completed".into()),
					("releasing".into(), "Releasing".into()),
					("on_hiatus".into(), "On Hiatus".into()),
					("discontinued".into(), "Discontinued".into()),
					("info".into(), "Not Yet Published".into()),
				],
				id: "status".into(),
			},
			Filter {
				name: "Language".into(),
				kind: FilterType::Select,
				values: vec![
					("".into(), "Any".into()),
					("en".into(), "English".into()),
					("fr".into(), "French".into()),
					("es".into(), "Spanish".into()),
					("es-la".into(), "Spanish (LATAM)".into()),
					("pt".into(), "Portuguese".into()),
					("pt-br".into(), "Portuguese (Br)".into()),
					("ja".into(), "Japanese".into()),
				],
				id: "language".into(),
			},
			Filter {
				name: "Year".into(),
				kind: FilterType::Select,
				values: (1930..=2025).rev().map(|y| (y.to_string(), y.to_string())).collect(),
				id: "year".into(),
			},
			Filter {
				name: "Length".into(),
				kind: FilterType::Select,
				values: vec![
					("".into(), "Any".into()),
					("1".into(), ">= 1 chapters".into()),
					("3".into(), ">= 3 chapters".into()),
					("5".into(), ">= 5 chapters".into()),
					("10".into(), ">= 10 chapters".into()),
					("20".into(), ">= 20 chapters".into()),
					("30".into(), ">= 30 chapters".into()),
					("50".into(), ">= 50 chapters".into()),
				],
				id: "minchap".into(),
			},
			Filter {
				name: "Sort by".into(),
				kind: FilterType::Select,
				values: vec![
					("recently_updated".into(), "Recently updated".into()),
					("recently_added".into(), "Recently added".into()),
					("release_date".into(), "Release date".into()),
					("trending".into(), "Trending".into()),
					("title_az".into(), "Name A-Z".into()),
					("scores".into(), "Scores".into()),
					("mal_scores".into(), "MAL scores".into()),
					("most_viewed".into(), "Most viewed".into()),
					("most_favourited".into(), "Most favourited".into()),
				],
				id: "sort".into(),
			},
		]
	}

	fn get_search_manga_list(&self, query: Option<String>, page: i32, filters: Vec<FilterValue>) -> Result<MangaPageResult> {
		let mut url = format!("{}/filter?page={}", BASE_URL, page);
		if let Some(q) = &query {
			url.push_str(&format!("&keyword={}", q));
		}
		for filter in filters {
			match filter {
				FilterValue::Select { id, value } => {
					if !value.is_empty() {
						url.push_str(&format!("&{}={}", id, value));
					}
				},
				FilterValue::MultiSelect { id, included, .. } => {
					if !included.is_empty() {
						for v in included {
							url.push_str(&format!("&{}[]={}", id, v));
						}
					}
				},
				_ => {}
			}
		}
		let html = Request::get(&url)?.html()?;
		let entries = html.select(".unit").map(|els| {
			els.filter_map(|el| {
				let manga_url = el.select_first("a.poster").and_then(|a| a.attr("href"))?;
				let key = manga_url.strip_prefix("/manga/")?.to_string();
				let title = el.select_first(".info > a").and_then(|a| a.text()).unwrap_or_default();
				let cover = el.select_first("a.poster img").and_then(|img| img.attr("src")).map(|s| s.to_string());
				Some(Manga {
					key,
					title,
					cover,
					..Default::default()
				})
			}).collect::<Vec<Manga>>()
		}).unwrap_or_default();
		let has_next_page = html.select_first(".pagination .page-item.active").is_some();
		Ok(MangaPageResult { entries, has_next_page })
	}

	fn get_manga_update(&self, mut manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
		let url = format!("{}/manga/{}", BASE_URL, manga.key);
		let html = Request::get(&url)?.html()?;
		if needs_details {
			manga.title = html.select_first(".info h1").and_then(|el| el.text()).unwrap_or_default();
			manga.cover = html.select_first(".poster img").and_then(|img| img.attr("src")).map(|s| s.to_string());
			manga.description = html.select_first(".summary").and_then(|el| el.text());
			manga.status = match html.select_first(".meta .status span").and_then(|el| el.text()).unwrap_or("") {
				"Completed" => MangaStatus::Completed,
				"Releasing" => MangaStatus::Ongoing,
				"On Hiatus" => MangaStatus::Hiatus,
				"Discontinued" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			};
			manga.authors = html.select(".meta .author a").map(|els| {
				els.filter_map(|el| el.text()).collect::<Vec<String>>()
			});
			manga.tags = html.select(".meta .genres a").map(|els| {
				els.filter_map(|el| el.text()).collect::<Vec<String>>()
			});
			let tags = manga.tags.as_deref().unwrap_or(&[]);
			manga.content_rating = if tags.iter().any(|e| matches!(e.as_str(), "Adult" | "Mature" | "Smut")) {
				ContentRating::NSFW
			} else if tags.iter().any(|e| e == "Ecchi") {
				ContentRating::Suggestive
			} else {
				ContentRating::Safe
			};
			manga.viewer = if tags.iter().any(|e| e == "Manga") {
				Viewer::RightToLeft
			} else if tags.iter().any(|e| matches!(e.as_str(), "Manhwa" | "Manhua")) {
				Viewer::Webtoon
			} else {
				Viewer::Unknown
			};
		}
		if needs_chapters {
			manga.chapters = html.select(".chapters li").map(|els| {
				els.filter_map(|el| {
					let a = el.select_first("a")?;
					let url = a.attr("href")?;
					let key = url.strip_prefix(&format!("/read/{}/", manga.key)).unwrap_or(url).to_string();
					let title = a.text();
					let date_uploaded = el.select_first("time").and_then(|t| t.attr("datetime")).and_then(|dt| chrono::DateTime::parse_from_rfc3339(dt).ok()).map(|d| d.timestamp());
					Some(Chapter {
						key,
						title,
						date_uploaded,
						url: Some(url.to_string()),
						..Default::default()
					})
				}).collect::<Vec<Chapter>>()
			});
		}
		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{}/read/{}/{}", BASE_URL, manga.key, chapter.key);
		let html = Request::get(&url)?.html()?;
		let mut pages = Vec::new();
		for img in html.select(".reader img").map(|els| els.filter_map(|el| el.attr("src")).collect::<Vec<&str>>()).unwrap_or_default() {
			pages.push(Page {
				content: PageContent::url(img.to_string()),
				..Default::default()
			});
		}
		Ok(pages)
	}
}

register_source!(MangafireSource);
