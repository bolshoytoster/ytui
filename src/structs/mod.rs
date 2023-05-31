//! Loads of structures related to JSON {,de}serialization because I like strong typing

#![allow(non_snake_case)]

pub mod comments;
pub mod continuation;
pub mod general;
pub mod next;
pub mod requests;
pub mod transcript;
pub mod video;

pub mod search;

use std::borrow::Cow;
use std::iter::once;

pub use comments::*;
pub use continuation::*;
pub use general::*;
pub use next::*;
use ratatui::style::{self, Modifier, Style};
use ratatui::text;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{ListItem, Paragraph, Wrap};
pub use requests::*;
pub use search::*;
use serde::Deserialize;
pub use transcript::*;
pub use video::*;

/// A selectable item
pub enum Node {
	/// A tag, property is the continuation token
	Header(String),
	/// A video, property is the video ID
	Video(String),
	/// A game, properties are the browse ID and the `params` field
	Game(String, Option<String>),
	/// A search, properties are the query and params
	Search(String, Option<String>),
	/// A channel, properties are the channel ID and params
	Channel(String, Option<String>),
	/// A playlist, property is the playlist ID
	Playlist(String),
	/// A video transcript
	Transcript(String),
	/// A video's comments, property is the continuation token
	CommentSection(String),
	/// A comment with replies, property is the token to get the first section of replies
	Comment(String),
	/// Can be hovered over, but does nothing
	None,
}

/// Turns an `Into<Spans>` into a ratatui `ListItem` with a line below for spacing
fn spaced<'a>(line: impl Into<Spans<'a>>) -> ListItem<'a> {
	ListItem::new(text::Text {
		lines: vec![line.into(), Spans::default()],
	})
}

/// Turns an `Into<Cow<str>>` into a ratatui `ListItem`, underlined
fn underlined<'a>(line: impl Into<Cow<'a, str>>) -> ListItem<'a> {
	ListItem::new(text::Text {
		lines: vec![Spans(vec![Span {
			content: line.into(),
			style: Style {
				add_modifier: Modifier::UNDERLINED,
				..Style::default()
			},
		}])],
	})
}

/// Interprets integers as ratatui colours, first byte is alpha (ignored), last three are rgb.
fn int_to_colour(colour: u32) -> style::Color {
	style::Color::Rgb(
		((colour & 0xFF0000) >> 16) as u8,
		((colour & 0xFF00) >> 8) as u8,
		colour as u8,
	)
}

const EMPTY_TEXT: text::Text = text::Text { lines: Vec::new() };

/// Trait for objects that can be added to widget `Vec`s.
pub trait IntoWidgets {
	/// Convert this into ratatui widgets and add them to the given `Vec`s. Returns a continuation
	/// token, if there is one.
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String>;
}

#[derive(Deserialize)]
struct AppendContinuationItemsAction<T: IntoWidgets> {
	continuationItems: Vec<T>, // Ignore `targetId`
}

#[derive(Deserialize)]
struct ContinueOnResponseReceivedAction<T: IntoWidgets> {
	#[serde(alias = "reloadContinuationItemsCommand")]
	appendContinuationItemsAction: AppendContinuationItemsAction<T>,
	// Ignore `clickTrackingParams`
}

#[derive(Deserialize)]
struct BasicColorPaletteData {
	/// 4 bytes, first is alpha, last 3 are rgb
	backgroundColor: Option<u32>,
	foregroundTitleColor: u32,
	foregroundBodyColor: Option<u32>,
}

#[derive(Deserialize)]
struct Color {
	basicColorPaletteData: BasicColorPaletteData,
}

#[derive(Deserialize)]
struct SimpleText {
	simpleText: String,
}

#[derive(Deserialize)]
struct Run {
	text: String,
	bold: Option<bool>,
	italics: Option<bool>,
}

#[derive(Deserialize)]
struct Text {
	runs: Vec<Run>,
}
impl Text {
	/// Converts this to a `Spans`, applying style, adding the given style
	fn with_style<'a>(self, style: Style) -> Spans<'a> {
		Spans(
			self.runs
				.into_iter()
				.map(|run| Span {
					content: run.text.into(),
					style: Style {
						add_modifier: if run.bold == Some(true) {
							style.add_modifier | Modifier::BOLD
						} else {
							style.add_modifier
						} | if run.italics == Some(true) {
							Modifier::ITALIC
						} else {
							Modifier::empty()
						},
						..style
					},
				})
				.collect::<Vec<Span>>(),
		)
	}

	/// Converts this to a `Spans`, applying style, adding the underlinde modifier
	fn underlined<'a>(self) -> Spans<'a> {
		self.with_style(Style {
			add_modifier: Modifier::UNDERLINED,
			..Style::default()
		})
	}
}
impl<'a> From<Text> for Spans<'a> {
	fn from(value: Text) -> Spans<'a> {
		Spans(
			value
				.runs
				.into_iter()
				.map(|run| Span {
					content: run.text.into(),
					style: Style {
						add_modifier: if run.bold == Some(true) {
							Modifier::BOLD
						} else {
							Modifier::empty()
						} | if run.italics == Some(true) {
							Modifier::ITALIC
						} else {
							Modifier::empty()
						},
						..Style::default()
					},
				})
				.collect(),
		)
	}
}

#[derive(Deserialize)]
pub struct RichItemRendererContent {
	videoRenderer: VideoRenderer,
}

#[derive(Deserialize)]
struct RichItemRenderer<T> {
	content: T, // Ignore `trackingParams
}

#[derive(Deserialize)]
pub struct RichItem<T> {
	richItemRenderer: RichItemRenderer<T>,
}
#[derive(Deserialize)]
struct MetadataBadgeRendererVideo {
	label: String, // Ignore `icon`, `style` and `trackingParams`
}

#[derive(Deserialize)]
struct MetadataBadgeRendererOwner {
	tooltip: String, // Ignore `icon`, `style`, `trackingParams` and `accessibilityData`
}

#[derive(Deserialize)]
struct Badge<T> {
	metadataBadgeRenderer: T,
}

#[derive(Deserialize)]
struct AccessibilityData {
	label: String,
}

#[derive(Deserialize)]
struct Accessibility {
	accessibilityData: AccessibilityData,
}

#[derive(Deserialize)]
struct AccessibleText {
	accessibility: Accessibility,
	// Ignore `simpleText`
}

/// Group of common text formats
#[derive(Deserialize)]
#[serde(untagged)]
enum ShortViewCountText {
	Video(AccessibleText),
	Stream(Text),
}
impl<'a> From<ShortViewCountText> for Spans<'a> {
	fn from(value: ShortViewCountText) -> Spans<'a> {
		match value {
			ShortViewCountText::Video(accessible_text) => {
				accessible_text.accessibility.accessibilityData.label.into()
			}
			ShortViewCountText::Stream(text) => text.into(),
		}
	}
}

#[derive(Deserialize)]
struct VideoRenderer {
	badges: Option<Vec<Badge<MetadataBadgeRendererVideo>>>,
	descriptionSnippet: Option<Text>,
	lengthText: Option<AccessibleText>,
	ownerBadges: Option<Vec<Badge<MetadataBadgeRendererOwner>>>,
	ownerText: Text,
	publishedTimeText: Option<SimpleText>,
	shortViewCountText: ShortViewCountText,
	title: Text,
	videoId: String,
	// Ignore `channelThumbnailSupportedRenderers`, `inlinePlaybackEndpoint`, `menu`,
	// `navigationEndpoint` `shortBylineText`, `showActionMenu`, `thumbnail`, `trackingParams` and
	// `viewCountText`
}
impl IntoWidgets for VideoRenderer {
	/// Adds this video to the given `Vec`s
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		// Title on the left
		titles.push(spaced(self.title));

		let mut lines = vec![
			// Uploader
			self.ownerText.into(),
			"".into(),
			// View count, we have to handle this differently for streams and videos
			self.shortViewCountText.into(),
			"".into(),
		];

		// Description (not available for streams)
		if let Some(description_snippet) = self.descriptionSnippet {
			lines.extend([description_snippet.into(), "".into()]);
		}

		// Video length (not available for streams)
		if let Some(length_text) = self.lengthText {
			lines.push(length_text.accessibility.accessibilityData.label.into());
		}

		// Published time (not available for streams
		if let Some(published_time_text) = self.publishedTimeText {
			lines.push(published_time_text.simpleText.into())
		}

		// Video badges, if any
		if let Some(badges) = self.badges {
			lines.push(
				[
					"Badges: ",
					&badges
						.into_iter()
						.map(|badge| badge.metadataBadgeRenderer.label)
						.collect::<Vec<String>>()
						.join(", "),
				]
				.concat()
				.into(),
			);
		}

		// Uploader's badges
		if let Some(owner_badges) = self.ownerBadges {
			lines.push(
				[
					"Owner badges: ",
					&owner_badges
						.into_iter()
						.map(|badge| badge.metadataBadgeRenderer.tooltip)
						.collect::<Vec<String>>()
						.join(", "),
				]
				.concat()
				.into(),
			)
		}

		info.push((
			Paragraph::new(lines).wrap(Wrap { trim: false }),
			Node::Video(self.videoId),
		));

		None
	}
}

#[derive(Deserialize)]
struct BrowseEndpoint {
	browseId: String,
	params: Option<String>,
	// Ignore `apiUrl`
}

#[derive(Deserialize)]
struct Endpoint {
	browseEndpoint: BrowseEndpoint, // Ignore `clickTrackingParams`, `commandMetadata`
}

#[derive(Deserialize)]
struct ReelItemRenderer {
	headline: SimpleText,
	videoId: String,
	viewCountText: AccessibleText,
	// Ignore `accessibility`, `loggingDirectives`, `menu`, `navigationEndpoint`, `style`,
	// `thumbnail`, `trackingParams`, `videoType`
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RichSectionItemRendererContent {
	Video(RichItemRendererContent),
	ReelItem { reelItemRenderer: ReelItemRenderer },
	Game { gameCardRenderer: GameCardRenderer },
}
impl IntoWidgets for RichSectionItemRendererContent {
	fn into_widgets<'a>(
		self,
		list: &mut Vec<ListItem<'a>>,
		info_vec: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			RichSectionItemRendererContent::Video(rich_item_renderer_content) => {
				rich_item_renderer_content
					.videoRenderer
					.into_widgets(list, info_vec);
			}
			RichSectionItemRendererContent::ReelItem { reelItemRenderer } => {
				list.push(spaced(reelItemRenderer.headline.simpleText));

				info_vec.push((
					Paragraph::new(
						reelItemRenderer
							.viewCountText
							.accessibility
							.accessibilityData
							.label,
					)
					.wrap(Wrap { trim: false }),
					Node::Video(reelItemRenderer.videoId),
				));
			}
			RichSectionItemRendererContent::Game { gameCardRenderer } => {
				gameCardRenderer
					.game
					.gameDetailsRenderer
					.into_widgets(list, info_vec);
			}
		}

		None
	}
}

#[derive(Deserialize)]
struct RichShelfRenderer {
	contents: Vec<RichItem<RichSectionItemRendererContent>>,
	endpoint: Option<Endpoint>,
	subtitle: Option<Text>,
	title: Text,
	// Ignore `icon`, `menu`, `showMoreButton`, `trackingParams`
}

#[derive(Deserialize)]
struct RichSectionRendererContent {
	richShelfRenderer: RichShelfRenderer,
}

#[derive(Deserialize)]
pub struct RichSectionRenderer {
	content: RichSectionRendererContent, // Ignore `fullBleed` and `trackingParams`
}

#[derive(Deserialize)]
pub struct GridVideoRenderer {
	badges: Option<Vec<Badge<MetadataBadgeRendererVideo>>>,
	ownerBadges: Option<Vec<Badge<MetadataBadgeRendererOwner>>>,
	publishedTimeText: Option<SimpleText>,
	shortBylineText: Text,
	shortViewCountText: Option<ShortViewCountText>,
	title: Text,
	videoId: String,
	// Ignore `menu`, `navigationEndpoint`, `richThumbnail`, `thumbnail`, `thumbnailOverlays`,
	// `trackingParams` and `viewCountText`
}

#[derive(Deserialize)]
struct GameDetailsRenderer {
	title: SimpleText,
	endpoint: Endpoint,
	liveViewersText: Option<AccessibleText>,
	// Ignore `boxArt`, `boxArtOverlayText`, `trackingParams`, `isOfficialBoxArt`
}
impl IntoWidgets for GameDetailsRenderer {
	fn into_widgets<'a>(
		self,
		list: &mut Vec<ListItem<'a>>,
		info_vec: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		list.push(spaced(self.title.simpleText));

		info_vec.push((
			Paragraph::new(if let Some(live_viewers_text) = self.liveViewersText {
				live_viewers_text
					.accessibility
					.accessibilityData
					.label
					.into()
			} else {
				EMPTY_TEXT
			})
			.wrap(Wrap { trim: false }),
			Node::Game(
				self.endpoint.browseEndpoint.browseId,
				self.endpoint.browseEndpoint.params,
			),
		));

		None
	}
}

#[derive(Deserialize)]
struct VideoCardRenderer {
	lengthText: Option<AccessibleText>,
	metadataText: SimpleText,
	ownerBadges: Option<Vec<Badge<MetadataBadgeRendererOwner>>>,
	title: Text,
	videoId: String,
	// Ignore `bylineText`, `menu`, `navigationEndpoint`, `thumbnail`, `thumbnailOverlays` and
	// `trackingParams`
}

#[derive(Deserialize)]
struct SearchEndpoint {
	query: String,
	/// `None` for already selected filters
	params: Option<String>,
}

#[derive(Deserialize)]
struct SearchFilterRendererNavigationEndpoint {
	searchEndpoint: SearchEndpoint,
	// Ignore `clickTrackingParams` and `commandMetadata`
}

#[derive(Deserialize)]
struct SearchRefinementCardRenderer {
	query: Text,
	searchEndpoint: SearchFilterRendererNavigationEndpoint,
	// Ignore `thumbnail` and `trackingParams`
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Card {
	Video {
		videoCardRenderer: VideoCardRenderer,
	},
	Game {
		gameCardRenderer: GameCardRenderer,
	},
	SearchRefinementCard {
		searchRefinementCardRenderer: SearchRefinementCardRenderer,
	},
}

#[derive(Deserialize)]
struct RichListHeaderRenderer {
	title: Title,
	subtitle: Option<Text>,
	// Ignore `trackingParams`, `titleStyle`
}

#[derive(Deserialize)]
struct Header {
	richListHeaderRenderer: RichListHeaderRenderer,
}

#[derive(Deserialize)]
struct HorizontalCardListRenderer {
	cards: Vec<Card>,
	header: Header,
	// Ignore `button`, `nextButton`, `previousButton`, `style` and `trackingParams`
}
impl HorizontalCardListRenderer {
	/// Add this item/category to the given lists
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) {
		titles.push(ListItem::new(
			self.header.richListHeaderRenderer.title.underlined(),
		));
		info.push((
			Paragraph::new(
				if let Some(subtitle) = self.header.richListHeaderRenderer.subtitle {
					subtitle.into()
				} else {
					Spans(Vec::new())
				},
			)
			.wrap(Wrap { trim: false }),
			Node::None,
		));

		for card in self.cards {
			match card {
				Card::Video { videoCardRenderer } => {
					titles.push(spaced(videoCardRenderer.title));

					let mut lines =
						vec![videoCardRenderer.metadataText.simpleText.into(), "".into()];

					if let Some(length_text) = videoCardRenderer.lengthText {
						lines.push(
							[
								"Length: ",
								&length_text.accessibility.accessibilityData.label,
							]
							.concat()
							.into(),
						);
					}

					if let Some(owner_badges) = videoCardRenderer.ownerBadges {
						lines.push(
							[
								"Badges: ",
								&owner_badges
									.into_iter()
									.map(|badge| badge.metadataBadgeRenderer.tooltip)
									.collect::<String>(),
							]
							.concat()
							.into(),
						);
					}

					info.push((
						Paragraph::new(lines).wrap(Wrap { trim: false }),
						Node::Video(videoCardRenderer.videoId),
					));
				}
				Card::Game { gameCardRenderer } => {
					gameCardRenderer
						.game
						.gameDetailsRenderer
						.into_widgets(titles, info);
				}
				Card::SearchRefinementCard {
					searchRefinementCardRenderer,
				} => {
					titles.push(ListItem::new(Spans::from(
						searchRefinementCardRenderer.query,
					)));
					info.push((
						Paragraph::new(EMPTY_TEXT).wrap(Wrap { trim: false }),
						Node::Search(
							searchRefinementCardRenderer
								.searchEndpoint
								.searchEndpoint
								.query,
							None,
						),
					));
				}
			}
		}
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Title {
	ReelShelfRenderer(Text),
	ItemSectionRenderer(SimpleText),
}
impl Title {
	/// Converts this to a `Spans` with the underlined modifier
	fn underlined<'a>(self) -> Spans<'a> {
		match self {
			Title::ReelShelfRenderer(text) => text.underlined(),
			Title::ItemSectionRenderer(simple_text) => Spans(vec![Span {
				content: simple_text.simpleText.into(),
				style: Style {
					add_modifier: Modifier::UNDERLINED,
					..Style::default()
				},
			}]),
		}
	}
}
impl<'a> From<Title> for Spans<'a> {
	fn from(value: Title) -> Spans<'a> {
		match value {
			Title::ReelShelfRenderer(text) => text.into(),
			Title::ItemSectionRenderer(simple_text) => simple_text.simpleText.into(),
		}
	}
}

#[derive(Deserialize)]
struct ReelShelfRenderer {
	items: Vec<RichSectionItemRendererContent>,
	title: Title, // Ignore `endpoint`, `icon` and `trackingParams`
}
impl ReelShelfRenderer {
	/// Add this item/category to the given lists
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) {
		titles.push(ListItem::new(self.title.underlined()));
		info.push((Paragraph::new(EMPTY_TEXT), Node::None));

		for item in self.items {
			item.into_widgets(titles, info);
		}
	}
}

#[derive(Deserialize)]
struct GridRenderer {
	items: Vec<RichGridRendererContent>, // Ignore `isCollapsible`, `targetId` and `trackingParams`
}

#[derive(Deserialize)]
struct ExpandedShelfContentsRenderer {
	items: Vec<RichItemRendererContent>,
}

#[derive(Deserialize)]
struct DetailedMetadataSnippet {
	snippetText: Text,
	// Ignore `snippetHoverText` and `maxOneLine`
}

#[derive(Deserialize)]
struct VerticalListRendererItemVideoRenderer {
	videoId: String,
	title: Text,
	publishedTimeText: Option<SimpleText>,
	lengthText: Option<AccessibleText>,
	badges: Option<Vec<Badge<MetadataBadgeRendererVideo>>>,
	ownerBadges: Option<Vec<Badge<MetadataBadgeRendererOwner>>>,
	ownerText: Text,
	shortViewCountText: ShortViewCountText,
	detailedMetadataSnippets: Option<Vec<DetailedMetadataSnippet>>,
	// Ignore `thumbnail`, `longBylineText`, `viewCountText`, `navigationEndpoint`,
	// `shortBylineText`, `trackingParams`, `showActionMenu`, `menu`,
	// `channelThumbnailSupportedRenderers`, `thumbnailOverlays` and `searchVideoResultEntityKey`
}

#[derive(Deserialize)]
struct VerticalListRendererItem {
	videoRenderer: VerticalListRendererItemVideoRenderer,
}
impl VerticalListRendererItem {
	/// Add this item/category to the given lists
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) {
		titles.push(spaced(self.videoRenderer.title));

		let mut lines = vec![
			self.videoRenderer.ownerText.into(),
			"".into(),
			self.videoRenderer.shortViewCountText.into(),
		];

		// Upload date (not on streams)
		if let Some(published_time_text) = self.videoRenderer.publishedTimeText {
			lines.push(published_time_text.simpleText.into());
		}

		// Video length (not on streams)
		if let Some(length_text) = self.videoRenderer.lengthText {
			lines.push(length_text.accessibility.accessibilityData.label.into());
		}

		if let Some(badges) = self.videoRenderer.badges {
			lines.push(
				[
					"Badges: ",
					&badges
						.into_iter()
						.map(|badge| badge.metadataBadgeRenderer.label)
						.collect::<Vec<String>>()
						.join(", "),
				]
				.concat()
				.into(),
			);
		}

		if let Some(owner_badges) = self.videoRenderer.ownerBadges {
			lines.push(
				[
					"Owner badges: ",
					&owner_badges
						.into_iter()
						.map(|owner_badge| owner_badge.metadataBadgeRenderer.tooltip)
						.collect::<Vec<String>>()
						.join(", "),
				]
				.concat()
				.into(),
			);
		}

		// Description
		if let Some(detailed_metadata_snippets) = self.videoRenderer.detailedMetadataSnippets {
			lines.extend(
				once(Spans(Vec::new())).chain(
					detailed_metadata_snippets
						.into_iter()
						.map(|detailed_metadata_snippet| {
							detailed_metadata_snippet.snippetText.into()
						}),
				),
			);
		}

		info.push((
			Paragraph::new(lines).wrap(Wrap { trim: false }),
			Node::Video(self.videoRenderer.videoId),
		));
	}
}

#[derive(Deserialize)]
struct VerticalListRenderer {
	items: Vec<VerticalListRendererItem>,
	// Ignore `collapsedItemCount`, `collapsedStateButtonText` and `trackingParams`
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ShelfRendererContent {
	Grid {
		gridRenderer: GridRenderer,
	},
	ExpandedShelfContents {
		expandedShelfContentsRenderer: ExpandedShelfContentsRenderer,
	},
	VerticalList {
		verticalListRenderer: VerticalListRenderer,
	},
}

#[derive(Deserialize)]
struct ShelfRenderer {
	content: ShelfRendererContent,
	subtitle: Option<Text>,
	title: Option<Title>, // Ignore `sortFilter`, `trackingParams`
}
impl ShelfRenderer {
	/// Add this item/category to the given lists, returning continuation token if there is one
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		// Title, if there is one
		if let Some(title) = self.title {
			titles.push(ListItem::new(title.underlined()));
			info.push((
				Paragraph::new(if let Some(subtitle) = self.subtitle {
					subtitle.into()
				} else {
					Spans(Vec::new())
				})
				.wrap(Wrap { trim: false }),
				Node::None,
			));
		}

		match self.content {
			ShelfRendererContent::Grid { gridRenderer } => {
				for item in gridRenderer.items {
					if let Some(continuation_token) = item.into_widgets(titles, info) {
						return Some(continuation_token);
					}
				}
			}
			ShelfRendererContent::ExpandedShelfContents {
				expandedShelfContentsRenderer,
			} => {
				for item in expandedShelfContentsRenderer.items {
					item.videoRenderer.into_widgets(titles, info);
				}
			}
			ShelfRendererContent::VerticalList {
				verticalListRenderer,
			} => {
				for item in verticalListRenderer.items {
					item.into_widgets(titles, info);
				}
			}
		}

		None
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ItemSectionRendererContent {
	HorizontalCardList {
		horizontalCardListRenderer: HorizontalCardListRenderer,
	},
	ReelShelfRenderer {
		reelShelfRenderer: ReelShelfRenderer,
	},
	ShelfRenderer {
		shelfRenderer: ShelfRenderer,
	},
}

#[derive(Deserialize)]
struct ItemSectionRenderer {
	contents: Vec<ItemSectionRendererContent>, // Ignore `targetId` and `trackingParams`
}

#[derive(Deserialize)]
struct Game {
	gameDetailsRenderer: GameDetailsRenderer,
}

#[derive(Deserialize)]
pub struct GameCardRenderer {
	game: Game, // Ignore `trackingParams`
}

#[derive(Deserialize)]
struct WatchEndpoint {
	videoId: String,
	// Ignore `watchEndpointSupportedOnesieConfig`
}

#[derive(Deserialize)]
struct CurrentVideoEndpoint {
	watchEndpoint: WatchEndpoint,
	// Ignore `clickTrackingParams` and `commandMetadata`
}

#[derive(Deserialize)]
struct ContinuationCommand {
	token: String,
	// Ignore `request` and `command`
}

#[derive(Deserialize)]
struct ContinuationEndpoint {
	continuationCommand: ContinuationCommand, // Ignore `clickTrackingParams` and `commandMetadata`
}

#[derive(Deserialize)]
struct SubMenuItem {
	title: String,
	serviceEndpoint: ContinuationEndpoint,
	// Ignore `selected` and `trackingParams`
}

#[derive(Deserialize)]
struct SortFilterSubMenuRenderer {
	subMenuItems: Vec<SubMenuItem>, // Ignore `icon`, `accessibility` and `trackingParams`
}

#[derive(Deserialize)]
struct Menu {
	sortFilterSubMenuRenderer: SortFilterSubMenuRenderer,
}

#[derive(Deserialize)]
pub struct ContinuationItemRenderer {
	continuationEndpoint: ContinuationEndpoint, // Ignore `trigger` and `ghostCards`
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RichGridRendererContent {
	/// A video
	RichItem(RichItem<RichItemRendererContent>),
	/// A section
	RichSection {
		richSectionRenderer: RichSectionRenderer,
	},
	/// Video in a grid
	GridVideoRenderer {
		gridVideoRenderer: GridVideoRenderer,
	},
	/// A game
	GameCard { gameCardRenderer: GameCardRenderer },
	/// The end, with continuation token
	ContinuationItem {
		continuationItemRenderer: ContinuationItemRenderer,
	},
}
impl IntoWidgets for RichGridRendererContent {
	/// Add this item/category to the given lists, returning continuation token if it's given
	fn into_widgets<'a>(
		self,
		list: &mut Vec<ListItem<'a>>,
		info_vec: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			RichGridRendererContent::RichItem(rich_item) => {
				rich_item
					.richItemRenderer
					.content
					.videoRenderer
					.into_widgets(list, info_vec);
			}
			RichGridRendererContent::RichSection {
				richSectionRenderer,
			} => {
				// Title
				list.push(ListItem::new(
					richSectionRenderer
						.content
						.richShelfRenderer
						.title
						.underlined(),
				));

				// Info is the subtitle if there is one
				info_vec.push((
					Paragraph::new(
						if let Some(subtitle) =
							richSectionRenderer.content.richShelfRenderer.subtitle
						{
							subtitle.into()
						} else {
							Spans(Vec::new())
						},
					)
					.wrap(Wrap { trim: false }),
					if let Some(endpoint) = richSectionRenderer.content.richShelfRenderer.endpoint {
						Node::Game(
							endpoint.browseEndpoint.browseId,
							endpoint.browseEndpoint.params,
						)
					} else {
						Node::None
					},
				));

				for content in richSectionRenderer.content.richShelfRenderer.contents {
					content
						.richItemRenderer
						.content
						.into_widgets(list, info_vec);
				}

				// Line at the end of section for separation
				list.push(ListItem::new(text::Text {
					lines: vec![Spans(Vec::new())],
				}));
				info_vec.push((Paragraph::new(EMPTY_TEXT), Node::None));
			}
			RichGridRendererContent::GridVideoRenderer { gridVideoRenderer } => {
				list.push(spaced(gridVideoRenderer.title));

				let mut lines = vec![gridVideoRenderer.shortBylineText.into(), "".into()];

				// Upload date
				if let Some(published_time_text) = gridVideoRenderer.publishedTimeText {
					lines.push(published_time_text.simpleText.into());
				}

				// Views
				if let Some(short_view_count_text) = gridVideoRenderer.shortViewCountText {
					lines.push(short_view_count_text.into());
				}

				// Video badges
				if let Some(badges) = gridVideoRenderer.badges {
					lines.push(
						[
							"Badges: ",
							&badges
								.into_iter()
								.map(|badge| badge.metadataBadgeRenderer.label)
								.collect::<Vec<String>>()
								.join(", "),
						]
						.concat()
						.into(),
					)
				}

				// Uploader badges
				if let Some(owner_badges) = gridVideoRenderer.ownerBadges {
					lines.push(
						[
							"Owner badges: ",
							&owner_badges
								.into_iter()
								.map(|badge| badge.metadataBadgeRenderer.tooltip)
								.collect::<Vec<String>>()
								.join(", "),
						]
						.concat()
						.into(),
					)
				}

				info_vec.push((
					Paragraph::new(lines).wrap(Wrap { trim: false }),
					Node::Video(gridVideoRenderer.videoId),
				));
			}
			RichGridRendererContent::GameCard { gameCardRenderer } => {
				gameCardRenderer
					.game
					.gameDetailsRenderer
					.into_widgets(list, info_vec);
			}
			RichGridRendererContent::ContinuationItem {
				continuationItemRenderer,
			} => {
				return Some(
					continuationItemRenderer
						.continuationEndpoint
						.continuationCommand
						.token,
				);
			}
		}

		None
	}
}
