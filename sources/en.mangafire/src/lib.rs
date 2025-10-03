#![no_std]
use aidoku::{
	alloc::{string::ToString, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent,
	Result, Source, Viewer,
};

const BASE_URL: &str = "https://mangafire.to";

pub struct MangafireSource;

impl Source for MangafireSource {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let mut url = format!("{}/filter?page={}", BASE_URL, page);
		if let Some(q) = &query {
			url.push_str(&format!("&keyword={}", q));
		}
		for filter in filters {
			match filter {
				FilterValue::Sort { id, index, .. } => {
					// Map sort index to sort value based on filters.json
					let sort_values = [
						"recently_updated",
						"recently_added",
						"name_az",
						"release_date",
						"most_viewed",
						"score",
					];
					if let Some(sort_value) = sort_values.get(index as usize) {
						url.push_str(&format!("&{}={}", id, sort_value));
					}
				}
				FilterValue::Select { id, value } => {
					if !value.is_empty() {
						url.push_str(&format!("&{}={}", id, value));
					}
				}
				FilterValue::MultiSelect { id, included, .. } => {
					if !included.is_empty() {
						for v in included {
							url.push_str(&format!("&{}[]={}", id, v));
						}
					}
				}
				_ => {}
			}
		}
		let html = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
			.header("Referer", BASE_URL)
			.html()?;
		let entries = html
			.select(".unit")
			.map(|els| {
				els.filter_map(|el| {
					let manga_url = el.select_first("a.poster").and_then(|a| a.attr("href"))?;
					let key = manga_url.strip_prefix("/manga/")?.to_string();
					let title = el
						.select_first(".info > a")
						.and_then(|a| a.text())
						.unwrap_or_default();
					let cover = el
						.select_first("a.poster img")
						.and_then(|img| img.attr("src"))
						.map(|s| s.to_string());
					Some(Manga {
						key,
						title,
						cover,
						..Default::default()
					})
				})
				.collect::<Vec<Manga>>()
			})
			.unwrap_or_default();
		let has_next_page = html.select_first(".pagination .page-item.active").is_some();
		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let url = format!("{}/manga/{}", BASE_URL, manga.key);
		let html = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
			.header("Referer", BASE_URL)
			.html()?;
		if needs_details {
			manga.title = html
				.select_first(".info h1")
				.and_then(|el| el.text())
				.unwrap_or_default();
			manga.cover = html
				.select_first(".poster img")
				.and_then(|img| img.attr("src"))
				.map(|s| s.to_string());
			manga.description = html.select_first(".summary").and_then(|el| el.text());
			manga.status = match html
				.select_first(".meta .status span")
				.and_then(|el| el.text())
				.unwrap_or_default()
				.as_str()
			{
				"Completed" => MangaStatus::Completed,
				"Releasing" => MangaStatus::Ongoing,
				"On Hiatus" => MangaStatus::Hiatus,
				"Discontinued" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			};
			manga.authors = html
				.select(".meta .author a")
				.map(|els| els.filter_map(|el| el.text()).collect::<Vec<String>>());
			manga.tags = html
				.select(".meta .genres a")
				.map(|els| els.filter_map(|el| el.text()).collect::<Vec<String>>());
			let tags = manga.tags.as_deref().unwrap_or(&[]);
			manga.content_rating = if tags
				.iter()
				.any(|e| matches!(e.as_str(), "Adult" | "Mature" | "Smut"))
			{
				ContentRating::NSFW
			} else if tags.iter().any(|e| e == "Ecchi") {
				ContentRating::Suggestive
			} else {
				ContentRating::Safe
			};
			manga.viewer = if tags.iter().any(|e| e == "Manga") {
				Viewer::RightToLeft
			} else if tags
				.iter()
				.any(|e| matches!(e.as_str(), "Manhwa" | "Manhua"))
			{
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
					let key = url
						.strip_prefix(&format!("/read/{}/", manga.key))
						.unwrap_or(&url)
						.to_string();
					let title = a.text();
					let date_uploaded = el
						.select_first("time")
						.and_then(|t| t.attr("datetime"))
						.and_then(|dt| chrono::DateTime::parse_from_rfc3339(&dt).ok())
						.map(|d| d.timestamp());
					Some(Chapter {
						key,
						title,
						date_uploaded,
						url: Some(url.to_string()),
						..Default::default()
					})
				})
				.collect::<Vec<Chapter>>()
			});
		}
		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{}/read/{}/{}", BASE_URL, manga.key, chapter.key);
		let html = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
			.header("Referer", &format!("{}/manga/{}", BASE_URL, manga.key))
			.html()?;

		// Helper function to extract JavaScript array values
		fn extract_js_array(content: &str, var_name: &str) -> Vec<String> {
			// Look for patterns like: var_name = [...] or var_name=[...]
			let patterns = [
				format!("{} = [", var_name),
				format!("{}=[", var_name),
				format!("{} =[", var_name),
				format!("{}= [", var_name),
			];

			for pattern in &patterns {
				if let Some(start_idx) = content.find(pattern) {
					let after_start = &content[start_idx + pattern.len()..];
					if let Some(end_idx) = after_start.find(']') {
						let slice = &after_start[..end_idx];
						return slice
							.split(',')
							.filter_map(|s| {
								let trimmed = s.trim();
								// Remove quotes and clean up
								if trimmed.len() >= 2 {
									let bytes = trimmed.as_bytes();
									if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
										|| (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
									{
										let unquoted = &trimmed[1..trimmed.len() - 1];
										return Some(unquoted.replace("\\/", "/"));
									}
								}
								if !trimmed.is_empty() {
									Some(trimmed.replace("\\/", "/"))
								} else {
									None
								}
							})
							.collect();
					}
				}
			}
			Vec::new()
		}

		// Try to extract images from JavaScript first
		let script_content = html
			.select("script")
			.map(|els| els.filter_map(|e| e.data()).collect::<Vec<_>>().join("\n"))
			.unwrap_or_default();

		if !script_content.is_empty() {
			// Try common variable names used by manga sites
			let image_urls = extract_js_array(&script_content, "images");
			if !image_urls.is_empty() {
				return Ok(image_urls
					.iter()
					.map(|url| Page {
						content: PageContent::url(url.clone()),
						..Default::default()
					})
					.collect());
			}

			// Try alternative variable names
			let image_urls = extract_js_array(&script_content, "pageImages");
			if !image_urls.is_empty() {
				return Ok(image_urls
					.iter()
					.map(|url| Page {
						content: PageContent::url(url.clone()),
						..Default::default()
					})
					.collect());
			}

			// Try chapterImages pattern
			let image_urls = extract_js_array(&script_content, "chapterImages");
			if !image_urls.is_empty() {
				// Check if we need a CDN prefix
				let cdn_urls = extract_js_array(&script_content, "cdns");
				if let Some(cdn) = cdn_urls.first() {
					return Ok(image_urls
						.iter()
						.map(|path| Page {
							content: PageContent::url(format!("{}/{}", cdn, path)),
							..Default::default()
						})
						.collect());
				} else {
					return Ok(image_urls
						.iter()
						.map(|url| Page {
							content: PageContent::url(url.clone()),
							..Default::default()
						})
						.collect());
				}
			}
		}

		// Fallback: Try to get images from the reader HTML
		let pages = html
			.select(".reader img, #reader img, .read-content img")
			.map(|els| {
				els.filter_map(|el| {
					// Try data-src first (for lazy-loaded images), then src
					let img_url = el
						.attr("data-src")
						.or_else(|| el.attr("src"))
						.or_else(|| el.attr("data-url"))?;
					Some(Page {
						content: PageContent::url(img_url.to_string()),
						..Default::default()
					})
				})
				.collect()
			})
			.unwrap_or_default();

		Ok(pages)
	}
}

register_source!(MangafireSource);
