//! Structs returned in search responses

use ratatui::style::{Modifier, Style};
use ratatui::text::{self, Span, Spans};
use ratatui::widgets::{ListItem, Paragraph, Wrap};
use serde::Deserialize;

use super::{
	int_to_colour, spaced, underlined, AccessibleText, Color, ContinuationItemRenderer,
	ContinueOnResponseReceivedAction, CurrentVideoEndpoint, Endpoint, HorizontalCardListRenderer,
	IntoWidgets, Node, ReelShelfRenderer, SearchFilterRendererNavigationEndpoint, ShelfRenderer,
	SimpleText, Text, VerticalListRendererItem, EMPTY_TEXT,
};

#[derive(Deserialize)]
struct DidYouMeanRenderer {
	correctedQuery: Text,
	correctedQueryEndpoint: SearchFilterRendererNavigationEndpoint,
	// Ignore `didYouMean` and `trackingParams`
}

#[derive(Deserialize)]
struct ChannelRenderer {
	channelId: String,
	title: SimpleText,
	descriptionSnippet: Text,
	// Ignore `navigationEndpoint`, `thumbnail`
}

#[derive(Deserialize)]
struct ChildVideoRenderer {
	title: SimpleText,
	lengthText: AccessibleText,
	videoId: String, // Ignore `navigationEndpoint`
}

#[derive(Deserialize)]
struct Video {
	childVideoRenderer: ChildVideoRenderer,
}

#[derive(Deserialize)]
struct PlaylistRenderer {
	playlistId: String,
	title: SimpleText,
	videos: Vec<Video>,
	thumbnailText: Text,
	longBylineText: Text,
	// Ignore `thumbnails`, `videoCount`, `navigationEndpoint`, `viewPlaylistText`,
	// `shortBylineText`, `viewCountText`, `trackingParams`, `thumbnailRenderer` and
	// `thumbnailOverlays`
}

#[derive(Deserialize)]
struct BackgroundPromoRenderer {
	title: Text,
	bodyText: Text,
	// Ignore `icon`, `trackingParams` and `style`
}

/// Ignore ads
#[derive(Deserialize)]
struct AdSlotRenderer {}

#[derive(Deserialize)]
#[serde(untagged)]
enum SectionListRendererContentItemSectionRendererContent {
	DidYouMean {
		didYouMeanRenderer: DidYouMeanRenderer,
	},
	Channel {
		channelRenderer: ChannelRenderer,
	},
	Video(VerticalListRendererItem),
	ReelShelf {
		reelShelfRenderer: ReelShelfRenderer,
	},
	Shelf {
		shelfRenderer: ShelfRenderer,
	},
	Playlist {
		playlistRenderer: PlaylistRenderer,
	},
	HorizontalCardList {
		horizontalCardListRenderer: HorizontalCardListRenderer,
	},
	BackgroundPromo {
		backgroundPromoRenderer: BackgroundPromoRenderer,
	},
	/// Ads, Ignore them
	AdSlot {
		#[allow(dead_code)]
		adSlotRenderer: AdSlotRenderer,
	},
}

#[derive(Deserialize)]
struct SectionListRendererContentItemSectionRenderer {
	contents: Vec<SectionListRendererContentItemSectionRendererContent>,
	// Ignore `trackingParams`
}
impl SectionListRendererContentItemSectionRenderer {
	/// Convert this into widgets
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) {
		for content in self.contents {
			match content {
				SectionListRendererContentItemSectionRendererContent::Video(
					vertical_list_renderer_item,
				) => vertical_list_renderer_item.into_widgets(titles, info),
				// Ignore continuation token
				SectionListRendererContentItemSectionRendererContent::Shelf { shelfRenderer } => {
					shelfRenderer.into_widgets(titles, info);
				}
				SectionListRendererContentItemSectionRendererContent::Playlist {
					playlistRenderer,
				} => {
					titles.push(spaced(playlistRenderer.title.simpleText));
					info.push((
						Paragraph::new(vec![
							playlistRenderer.longBylineText.into(),
							playlistRenderer.thumbnailText.into(),
						])
						.wrap(Wrap { trim: false }),
						Node::Playlist(playlistRenderer.playlistId),
					));

					// First few videos
					for video in playlistRenderer.videos {
						titles.push(ListItem::new(video.childVideoRenderer.title.simpleText));
						info.push((
							Paragraph::new(
								video
									.childVideoRenderer
									.lengthText
									.accessibility
									.accessibilityData
									.label,
							)
							.wrap(Wrap { trim: false }),
							Node::Video(video.childVideoRenderer.videoId),
						));
					}

					// Empty line
					titles.push(ListItem::new(text::Text {
						lines: vec![Spans(Vec::new())],
					}));
					info.push((Paragraph::new(EMPTY_TEXT), Node::None));
				}
				SectionListRendererContentItemSectionRendererContent::Channel {
					channelRenderer,
				} => {
					titles.push(spaced(channelRenderer.title.simpleText));
					info.push((
						Paragraph::new(Spans::from(channelRenderer.descriptionSnippet))
							.wrap(Wrap { trim: false }),
						Node::Channel(channelRenderer.channelId, None),
					));
				}
				SectionListRendererContentItemSectionRendererContent::ReelShelf {
					reelShelfRenderer,
				} => reelShelfRenderer.into_widgets(titles, info),
				SectionListRendererContentItemSectionRendererContent::DidYouMean {
					didYouMeanRenderer,
				} => {
					let mut title = Spans::from(didYouMeanRenderer.correctedQuery);

					title.0.insert(0, "Did you mean: ".into());

					titles.push(ListItem::new(title));
					info.push((
						Paragraph::new(EMPTY_TEXT),
						Node::Search(
							didYouMeanRenderer
								.correctedQueryEndpoint
								.searchEndpoint
								.query,
							didYouMeanRenderer
								.correctedQueryEndpoint
								.searchEndpoint
								.params,
						),
					));
				}
				SectionListRendererContentItemSectionRendererContent::BackgroundPromo {
					backgroundPromoRenderer,
				} => {
					titles.push(ListItem::new(Spans::from(backgroundPromoRenderer.title)));
					info.push((
						Paragraph::new(Spans::from(backgroundPromoRenderer.bodyText))
							.wrap(Wrap { trim: false }),
						Node::None,
					));
				}
				SectionListRendererContentItemSectionRendererContent::HorizontalCardList {
					horizontalCardListRenderer,
				} => horizontalCardListRenderer.into_widgets(titles, info),
				// Ignore ads
				SectionListRendererContentItemSectionRendererContent::AdSlot { .. } => (),
			}
		}
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SectionListRendererContent {
	ItemSection {
		itemSectionRenderer: SectionListRendererContentItemSectionRenderer,
	},
	ContinuationItem {
		continuationItemRenderer: ContinuationItemRenderer,
	},
}
impl IntoWidgets for SectionListRendererContent {
	/// Convert this into widgets, returning the continuation token if there is one
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			SectionListRendererContent::ItemSection {
				itemSectionRenderer,
			} => {
				itemSectionRenderer.into_widgets(titles, info);

				None
			}
			SectionListRendererContent::ContinuationItem {
				continuationItemRenderer,
			} => Some(
				continuationItemRenderer
					.continuationEndpoint
					.continuationCommand
					.token,
			),
		}
	}
}

#[derive(Deserialize)]
struct SearchFilterRenderer {
	label: SimpleText,
	/// Present on selected filters
	status: Option<String>,
	/// Not present on selected sort or unavailable filters
	navigationEndpoint: Option<SearchFilterRendererNavigationEndpoint>,
	tooltip: String,
	// Ignore `trackingParams`
}

#[derive(Deserialize)]
struct Filter {
	searchFilterRenderer: SearchFilterRenderer,
}

#[derive(Deserialize)]
struct SearchFilterGroupRenderer {
	filters: Vec<Filter>,
	title: SimpleText, // Ignore `trackingParams`
}

#[derive(Deserialize)]
struct Group {
	searchFilterGroupRenderer: SearchFilterGroupRenderer,
}

#[derive(Deserialize)]
struct SearchSubMenuRenderer {
	groups: Vec<Group>,
	// Ignore `button`, `title` and `trackingParams`
}

#[derive(Deserialize)]
struct SubMenu {
	searchSubMenuRenderer: SearchSubMenuRenderer,
}

#[derive(Deserialize)]
struct SectionListRenderer {
	contents: Vec<SectionListRendererContent>,
	subMenu: SubMenu,
	// Ignore `hideBottomSeparator`, `targetId` and `trackingParams`
}

#[derive(Deserialize)]
struct PrimaryContents {
	sectionListRenderer: SectionListRenderer,
}

#[derive(Deserialize)]
struct WatchCardHeroVideoRenderer {
	navigationEndpoint: CurrentVideoEndpoint,
	title: SimpleText,
	subtitle: SimpleText,
	lengthText: AccessibleText,
	// Ignore `trackingParams`, `thumbnailOverlays` and `heroImage`
}

#[derive(Deserialize)]
struct CallToAction {
	watchCardHeroVideoRenderer: WatchCardHeroVideoRenderer,
}

#[derive(Deserialize)]
struct WatchCardRichHeaderRenderer {
	title: SimpleText,
	titleNavigationEndpoint: Endpoint,
	subtitle: SimpleText,
	colorSupportedDatas: Color,
	// Ignore `trackingParams`, `darkThemeColorSupportedDatas` and `style`
	// TODO: we're ignoring the dark theme colours, should we add this as a config?
}

#[derive(Deserialize)]
struct UniversalWatchCardRendererHeader {
	watchCardRichHeaderRenderer: WatchCardRichHeaderRenderer,
}

#[derive(Deserialize)]
struct WatchCardCompactVideoRenderer {
	title: SimpleText,
	subtitle: SimpleText,
	lengthText: AccessibleText,
	navigationEndpoint: CurrentVideoEndpoint,
	// Ignore `thumbnail`, `trackingParams`, `thumbnailOverlays`, `byline`
}

#[derive(Deserialize)]
struct VerticalWatchCardListRendererItem {
	watchCardCompactVideoRenderer: WatchCardCompactVideoRenderer,
}

#[derive(Deserialize)]
struct VerticalWatchCardListRenderer {
	items: Vec<VerticalWatchCardListRendererItem>,
	viewAllEndpoint: Endpoint, // Ignore `trackingParams` and `viewAllText`
}

#[derive(Deserialize)]
struct List {
	verticalWatchCardListRenderer: VerticalWatchCardListRenderer,
}

#[derive(Deserialize)]
struct WatchCardSectionSequenceRenderer {
	lists: Vec<List>, // Ignore `trackingParams`
}

#[derive(Deserialize)]
struct UniversalWatchCardRendererSection {
	watchCardSectionSequenceRenderer: WatchCardSectionSequenceRenderer,
}

#[derive(Deserialize)]
struct UniversalWatchCardRenderer {
	callToAction: CallToAction,
	header: UniversalWatchCardRendererHeader,
	sections: Vec<UniversalWatchCardRendererSection>,
	// Ignore `collapsedLabel` and `trackingParams`
}

#[derive(Deserialize)]
struct SecondarySearchContainerRendererContent {
	universalWatchCardRenderer: UniversalWatchCardRenderer,
}

#[derive(Deserialize)]
struct SecondarySearchContainerRenderer {
	contents: Vec<SecondarySearchContainerRendererContent>, // Ignore `trackingParams`
}

#[derive(Deserialize)]
struct SecondaryContents {
	secondarySearchContainerRenderer: SecondarySearchContainerRenderer,
}

#[derive(Deserialize)]
struct TwoColumnSearchResultsRenderer {
	primaryContents: PrimaryContents,
	secondaryContents: Option<SecondaryContents>,
}

#[derive(Deserialize)]
struct SearchResponseContents {
	twoColumnSearchResultsRenderer: TwoColumnSearchResultsRenderer,
}

#[derive(Deserialize)]
pub struct SearchResponse {
	contents: SearchResponseContents,
	estimatedResults: String,
	refinements: Option<Vec<String>>,
	// Ignore `onResponseReceivedCommands`, `responseContext`, `topbar` and `targetId`
}
impl SearchResponse {
	pub fn into_widgets<'a>(
		self,
	) -> (
		Vec<ListItem<'a>>,
		Vec<(Paragraph<'a>, Node)>,
		Option<String>,
	) {
		// Total number of items, TODO
		let len = 0;

		let mut titles = Vec::with_capacity(len);
		let mut info = Vec::with_capacity(len);

		// Continuation token
		let mut continuation = None;

		// Number of results
		titles.push(spaced([&self.estimatedResults, " results"].concat()));
		info.push((Paragraph::new(EMPTY_TEXT), Node::None));

		// Search suggestions
		if let Some(refinements) = self.refinements {
			titles.push(underlined("Search suggestions"));
			info.push((Paragraph::new(EMPTY_TEXT), Node::None));

			for refinement in refinements {
				titles.push(ListItem::new(refinement.clone()));
				info.push((Paragraph::new(EMPTY_TEXT), Node::Search(refinement, None)));
			}

			// Empty line
			titles.push(ListItem::new(text::Text {
				lines: vec![Spans(Vec::new())],
			}));
			info.push((Paragraph::new(EMPTY_TEXT), Node::None));
		}

		// Filters
		titles.push(underlined("Filters"));
		info.push((Paragraph::new(EMPTY_TEXT), Node::None));

		for group in self
			.contents
			.twoColumnSearchResultsRenderer
			.primaryContents
			.sectionListRenderer
			.subMenu
			.searchSubMenuRenderer
			.groups
		{
			titles.push(underlined(group.searchFilterGroupRenderer.title.simpleText));
			info.push((Paragraph::new(EMPTY_TEXT), Node::None));

			for filter in group.searchFilterGroupRenderer.filters {
				titles.push(ListItem::new(Span {
					content: filter.searchFilterRenderer.label.simpleText.into(),
					style: Style {
						add_modifier: match filter.searchFilterRenderer.status.as_deref() {
							// Selected filters are bold
							Some("FILTER_STATUS_SELECTED") => Modifier::BOLD,
							// Unavailable filters have strikethrough
							Some("FILTER_STATUS_DISABLED") => Modifier::CROSSED_OUT,
							_ => Modifier::empty(),
						},
						..Style::default()
					},
				}));

				info.push((
					Paragraph::new(filter.searchFilterRenderer.tooltip).wrap(Wrap { trim: false }),
					if let Some(navigation_endpoint) =
						filter.searchFilterRenderer.navigationEndpoint
					{
						// Most filters
						Node::Search(
							navigation_endpoint.searchEndpoint.query,
							navigation_endpoint.searchEndpoint.params,
						)
					} else {
						// Selected sort
						Node::None
					},
				));
			}
		}

		// Empty line
		titles.push(ListItem::new(text::Text {
			lines: vec![Spans(Vec::new())],
		}));
		info.push((Paragraph::new(EMPTY_TEXT), Node::None));

		// The section on the right
		if let Some(secondary_contents) = self
			.contents
			.twoColumnSearchResultsRenderer
			.secondaryContents
		{
			for content in secondary_contents.secondarySearchContainerRenderer.contents {
				// Title
				titles.push(
					spaced(
						content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.title
							.simpleText,
					)
					.style(Style {
						fg: Some(int_to_colour(
							content
								.universalWatchCardRenderer
								.header
								.watchCardRichHeaderRenderer
								.colorSupportedDatas
								.basicColorPaletteData
								.foregroundTitleColor,
						)),
						bg: content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.colorSupportedDatas
							.basicColorPaletteData
							.backgroundColor
							.map(int_to_colour),
						..Style::default()
					}),
				);
				info.push((
					Paragraph::new(
						content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.subtitle
							.simpleText,
					)
					.style(Style {
						fg: content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.colorSupportedDatas
							.basicColorPaletteData
							.foregroundBodyColor
							.map(int_to_colour),
						bg: content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.colorSupportedDatas
							.basicColorPaletteData
							.backgroundColor
							.map(int_to_colour),
						..Style::default()
					})
					.wrap(Wrap { trim: false }),
					Node::Channel(
						content
							.universalWatchCardRenderer
							.header
							.watchCardRichHeaderRenderer
							.titleNavigationEndpoint
							.browseEndpoint
							.browseId,
						None,
					),
				));

				titles.push(spaced(
					content
						.universalWatchCardRenderer
						.callToAction
						.watchCardHeroVideoRenderer
						.title
						.simpleText,
				));
				info.push((
					Paragraph::new(vec![
						content
							.universalWatchCardRenderer
							.callToAction
							.watchCardHeroVideoRenderer
							.subtitle
							.simpleText
							.into(),
						"".into(),
						content
							.universalWatchCardRenderer
							.callToAction
							.watchCardHeroVideoRenderer
							.lengthText
							.accessibility
							.accessibilityData
							.label
							.into(),
					])
					.wrap(Wrap { trim: false }),
					Node::Video(
						content
							.universalWatchCardRenderer
							.callToAction
							.watchCardHeroVideoRenderer
							.navigationEndpoint
							.watchEndpoint
							.videoId,
					),
				));

				for section in content.universalWatchCardRenderer.sections {
					for list in section.watchCardSectionSequenceRenderer.lists {
						for item in list.verticalWatchCardListRenderer.items {
							titles
								.push(spaced(item.watchCardCompactVideoRenderer.title.simpleText));
							info.push((
								Paragraph::new(vec![
									item.watchCardCompactVideoRenderer
										.subtitle
										.simpleText
										.into(),
									"".into(),
									item.watchCardCompactVideoRenderer
										.lengthText
										.accessibility
										.accessibilityData
										.label
										.into(),
								])
								.wrap(Wrap { trim: false }),
								Node::Video(
									item.watchCardCompactVideoRenderer
										.navigationEndpoint
										.watchEndpoint
										.videoId,
								),
							));
						}

						// 'View all' from this section
						titles.push(ListItem::new("View all"));
						info.push((
							Paragraph::new(EMPTY_TEXT),
							Node::Channel(
								list.verticalWatchCardListRenderer
									.viewAllEndpoint
									.browseEndpoint
									.browseId,
								list.verticalWatchCardListRenderer
									.viewAllEndpoint
									.browseEndpoint
									.params,
							),
						));
					}
				}
			}

			// Empty line
			titles.push(ListItem::new(text::Text {
				lines: vec![Spans(Vec::new())],
			}));
			info.push((Paragraph::new(EMPTY_TEXT), Node::None));
		}

		// The main stuff
		for content in self
			.contents
			.twoColumnSearchResultsRenderer
			.primaryContents
			.sectionListRenderer
			.contents
		{
			match content {
				SectionListRendererContent::ItemSection {
					itemSectionRenderer,
				} => itemSectionRenderer.into_widgets(&mut titles, &mut info),
				SectionListRendererContent::ContinuationItem {
					continuationItemRenderer,
				} => {
					continuation = Some(
						continuationItemRenderer
							.continuationEndpoint
							.continuationCommand
							.token,
					)
				}
			}
		}

		(titles, info, continuation)
	}
}

type OnResponseReceivedCommands = Vec<ContinueOnResponseReceivedAction<SectionListRendererContent>>;

#[derive(Deserialize)]
pub struct SearchContinuationResponse {
	onResponseReceivedCommands: OnResponseReceivedCommands,
	// Ignore `responseContext`, `estimatedResults`, `trackingParams` and `topbar`
}
impl SearchContinuationResponse {
	/// Convert this into widgets, returning the continuation token if there is one
	pub fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		let mut continuation = None;

		for on_response_received_command in self.onResponseReceivedCommands {
			for continuation_item in on_response_received_command
				.appendContinuationItemsAction
				.continuationItems
			{
				if let Some(new_continuation) = continuation_item.into_widgets(titles, info) {
					continuation = Some(new_continuation);
				}
			}
		}

		continuation
	}
}
