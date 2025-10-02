# MangaFire (mangafire.to) Aidoku Extension

This is an Aidoku extension for the English manga site [mangafire.to](https://mangafire.to).

## Features
- Search manga with filters
- Fetch manga details (title, cover, description, status, authors, tags, content rating, viewer)
- List chapters with dates
- Get pages for a chapter (supports JavaScript-based and HTML-based image loading)

## Implementation Notes
- The `get_page_list` function extracts chapter images from both JavaScript variables and HTML elements
- Supports common variable names: `images`, `pageImages`, `chapterImages`
- Handles CDN URLs when present (e.g., `cdns` array)
- Falls back to HTML parsing if JavaScript extraction fails
- Properly handles lazy-loaded images with `data-src` attributes

## License
MIT/Apache-2.0
