#![no_std]
use aidoku::{
	prelude::*,
	alloc::{String, Vec, string::ToString},
	imports::net::Request,
	Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

const BASE_URL: &str = "https://mangafire.to";

pub struct MangafireSource;

impl Source for MangafireSource {
	fn new() -> Self {
		Self
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
			manga.status = match html.select_first(".meta .status span").and_then(|el| el.text()).unwrap_or_default().as_str() {
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
					let key = url.strip_prefix(&format!("/read/{}/", manga.key)).unwrap_or(&url).to_string();
					let title = a.text();
					let date_uploaded = el.select_first("time").and_then(|t| t.attr("datetime")).and_then(|dt| chrono::DateTime::parse_from_rfc3339(&dt).ok()).map(|d| d.timestamp());
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
		for img in html.select(".reader img").map(|els| els.filter_map(|el| el.attr("src")).collect::<Vec<String>>()).unwrap_or_default() {
			pages.push(Page {
				content: PageContent::url(img.to_string()),
				..Default::default()
			});
		}
		Ok(pages)
	}
}

register_source!(MangafireSource);
