//! Structures returned from requesting reccommendations for a video

#![allow(non_snake_case)]

use ratatui::style::Style;
use ratatui::text;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{ListItem, Paragraph, Wrap};
use serde::Deserialize;

use super::{
	int_to_colour, spaced, AccessibleText, Badge, CurrentVideoEndpoint, Endpoint, IntoWidgets,
	Menu, MetadataBadgeRendererOwner, MetadataBadgeRendererVideo, Node, ShortViewCountText,
	SimpleText, EMPTY_TEXT,
};

#[derive(Deserialize)]
pub struct CompactVideoRenderer {
	badges: Option<Vec<Badge<MetadataBadgeRendererVideo>>>,
	/// Not present on livestreams, obviously
	lengthText: Option<AccessibleText>,
	longBylineText: super::Text,
	ownerBadges: Option<Vec<Badge<MetadataBadgeRendererOwner>>>,
	publishedTimeText: Option<SimpleText>,
	shortViewCountText: ShortViewCountText,
	title: SimpleText,
	videoId: String,
	// Ignore `accessibility`, `channelThumbnail`, `navigationEndpoint, `menu`, `richThumbnail`,
	// `shortBylineText`, `thumbnail`, `thumbnailOverlays` and `trackingParams`
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SecondaryResultsResult {
	CompactVideo {
		compactVideoRenderer: CompactVideoRenderer,
	},
	ContinuationItem {
		continuationItemRenderer: super::ContinuationItemRenderer,
	},
}
impl IntoWidgets for SecondaryResultsResult {
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			SecondaryResultsResult::CompactVideo {
				compactVideoRenderer,
			} => {
				titles.push(spaced(compactVideoRenderer.title.simpleText));

				let mut lines = vec![
					compactVideoRenderer.longBylineText.into(),
					compactVideoRenderer.shortViewCountText.into(),
				];

				// These two are not present on streams
				if let Some(published_time_text) = compactVideoRenderer.publishedTimeText {
					lines.push(published_time_text.simpleText.into());
				}
				if let Some(length_text) = compactVideoRenderer.lengthText {
					lines.push(length_text.accessibility.accessibilityData.label.into());
				}

				if let Some(badges) = compactVideoRenderer.badges {
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

				if let Some(owner_badges) = compactVideoRenderer.ownerBadges {
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

				info.push((
					Paragraph::new(lines),
					Node::Video(compactVideoRenderer.videoId),
				));

				None
			}
			SecondaryResultsResult::ContinuationItem {
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
struct SecondaryResultsInner {
	results: Vec<SecondaryResultsResult>,
	// Ignore `targetId` and `trackingParams`
}

#[derive(Deserialize)]
struct SecondaryResultsOuter {
	secondaryResults: SecondaryResultsInner,
}

#[derive(Deserialize)]
struct TwoColumnWatchNextResults {
	secondaryResults: SecondaryResultsOuter,
	// Ignore `autoplay` and `results`
}

#[derive(Deserialize)]
struct NextResponseContents {
	twoColumnWatchNextResults: TwoColumnWatchNextResults,
}

#[derive(Deserialize)]
struct FactoidRenderer {
	accessibilityText: String, // Ignore `value` and `label`
}

#[derive(Deserialize)]
struct Factoid {
	factoidRenderer: FactoidRenderer,
}

#[derive(Deserialize)]
struct VideoDescriptionHeaderRenderer {
	title: super::Text,
	channel: SimpleText,
	factoid: Vec<Factoid>,
	channelNavigationEndpoint: Endpoint, // Ignore `views`, `publishDate` and `channelThumbnail`
}

#[derive(Deserialize)]
struct StyleRun {
	// Maximum 5000, so would fit in u16, but we need to use it for indexing
	startIndex: usize,
	length: usize,
	// 4 bytes, first is alpha, rest are rgb
	fontColor: u32,
}

#[derive(Deserialize)]
struct AttributedDescriptionBodyText {
	content: String,
	styleRuns: Vec<StyleRun>, // Ignore `commandRuns`
}

#[derive(Deserialize)]
struct ExpandableVideoDescriptionBodyRenderer {
	attributedDescriptionBodyText: AttributedDescriptionBodyText,
	// Ignore `showMoreText` and `showLessText`
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StructuredDescriptionContentItem {
	VideoDescriptionHeader {
		videoDescriptionHeaderRenderer: VideoDescriptionHeaderRenderer,
	},
	ExpandableVideoDescriptionBody {
		expandableVideoDescriptionBodyRenderer: ExpandableVideoDescriptionBodyRenderer,
	},
}

#[derive(Deserialize)]
struct StructuredDescriptionContentRenderer {
	items: Vec<StructuredDescriptionContentItem>,
}

#[derive(Deserialize)]
struct StructuredDescriptionContent {
	structuredDescriptionContentRenderer: StructuredDescriptionContentRenderer,
}

#[derive(Deserialize)]
struct EngagementPanelTitleHeaderRenderer {
	contextualInfo: super::Text,
	menu: Menu,
	// Ignore `visibilityButton` and `trackingParams`
}

#[derive(Deserialize)]
struct CommentsSectionHeader {
	engagementPanelTitleHeaderRenderer: EngagementPanelTitleHeaderRenderer,
}

#[derive(Deserialize)]
struct GetTranscriptEndpoint {
	params: String,
}

#[derive(Deserialize)]
struct ContinuationItemRendererContinuationEndpoint {
	getTranscriptEndpoint: GetTranscriptEndpoint,
	// Ignore `clickTrackingParams` and `commandMetadata`
}

#[derive(Deserialize)]
struct ContinuationItemRenderer {
	continuationEndpoint: ContinuationItemRendererContinuationEndpoint,
	// Ignore `trigger`
}

#[derive(Deserialize)]
struct SearchableTranscriptContent {
	continuationItemRenderer: ContinuationItemRenderer,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EngagementPanelSectionListRenderer {
	/// Title, views, published date, likes, description
	StructuredDescription {
		content: StructuredDescriptionContent,
		// Ignore `panelIdentifier`, `veType`, `targetId`, `visibility` and `loggingDirectives`
	},
	/// Comment number, different sorts
	CommentsSection { header: CommentsSectionHeader },
	/// The video transcript
	SearchableTranscript {
		content: SearchableTranscriptContent,
	},
	/// Something to do with ads, ignore it
	/// Can't be unit because of serde untagged stuff
	Ads {},
}

#[derive(Deserialize)]
struct EngagementPanel {
	engagementPanelSectionListRenderer: EngagementPanelSectionListRenderer,
}

#[derive(Deserialize)]
struct ThumbnailOverlayTimeStatusRenderer {
	text: AccessibleText,
}

#[derive(Deserialize)]
struct ThumbnailOverlay {
	thumbnailOverlayTimeStatusRenderer: ThumbnailOverlayTimeStatusRenderer,
}

#[derive(Deserialize)]
struct PlayerOverlayAutoplayRenderer {
	videoTitle: SimpleText,
	byline: super::Text,
	thumbnailOverlays: Vec<ThumbnailOverlay>,
	videoId: String,
	publishedTimeText: SimpleText,
	shortViewCountText: AccessibleText,
	// Ignore `title`, `pauseText`, `background`, `countDownSecs`, `cancelButton`,
	// `trackingParams`, `closeButton`, `preferImmediateRedirect`, `webShowNewAutonavCountdown`,
	// `webShowBigThumbnailEndscreen` and `countDownSecsForFullscreen`
}

#[derive(Deserialize)]
struct Autoplay {
	playerOverlayAutoplayRenderer: PlayerOverlayAutoplayRenderer,
}

#[derive(Deserialize)]
struct PlayerOverlayRenderer {
	autoplay: Autoplay,
	// Ignore `addToMenu`, `autonavToggle`, `decoratedPlayerBarRenderer`, `endScreen`,
	// `shareButton` and `videoDetails`
}

#[derive(Deserialize)]
struct PlayerOverlays {
	playerOverlayRenderer: PlayerOverlayRenderer,
}

#[derive(Deserialize)]
pub struct NextResponse {
	contents: NextResponseContents,
	currentVideoEndpoint: CurrentVideoEndpoint,
	engagementPanels: Vec<EngagementPanel>,
	playerOverlays: PlayerOverlays,
	// Ignore `frameworkUpdates`, `onResponseReceivedEndpoints`, `pageVisualEffects`,
	// `responseContext`, `topbar` and `trackingParams`
}
impl NextResponse {
	pub fn into_widgets<'a>(
		self,
	) -> (
		Vec<ListItem<'a>>,
		Vec<(Paragraph<'a>, Node)>,
		Option<String>,
	) {
		// Total number of items
		let len = self
			.contents
			.twoColumnWatchNextResults
			.secondaryResults
			.secondaryResults
			.results
			.len() + 6;

		let mut titles = Vec::with_capacity(len);
		let mut info = Vec::with_capacity(len);

		// Continuation token
		let mut continuation = None;

		for engagement_panels in self.engagementPanels {
			match engagement_panels.engagementPanelSectionListRenderer {
				// Video info
				EngagementPanelSectionListRenderer::StructuredDescription { content } => {
					// Description is in a different object to the rest of the info, so we need
					// this.
					let mut lines = Vec::with_capacity(5);

					for item in content.structuredDescriptionContentRenderer.items {
						match item {
							StructuredDescriptionContentItem::VideoDescriptionHeader {
								videoDescriptionHeaderRenderer,
							} => {
								titles.push(spaced(videoDescriptionHeaderRenderer.title));

								// Channel name
								lines
									.push(videoDescriptionHeaderRenderer.channel.simpleText.into());
								lines.extend(
									videoDescriptionHeaderRenderer.factoid.into_iter().map(
										|factoid| factoid.factoidRenderer.accessibilityText.into(),
									),
								);
							}
							StructuredDescriptionContentItem::ExpandableVideoDescriptionBody {
								expandableVideoDescriptionBodyRenderer,
							} => {
								let mut chars = expandableVideoDescriptionBodyRenderer
									.attributedDescriptionBodyText
									.content
									.chars();

								let mut current_line = Spans(Vec::with_capacity(1));

								let mut index = 0;

								// For each section
								for style_run in expandableVideoDescriptionBodyRenderer
									.attributedDescriptionBodyText
									.styleRuns
								{
									let section = (&mut chars)
										.take(style_run.startIndex - index)
										.collect::<String>();
									let mut section_lines = section.lines();

									// Add the text before this section, if there is any.
									if let Some(next) = section_lines.next() {
										// First line is added separately
										current_line.0.push(next.to_owned().into());

										for line in section_lines {
											lines.push(current_line);
											current_line = Spans(vec![line.to_owned().into()]);
										}
									}

									current_line.0.push(Span {
										content: (&mut chars)
											.take(style_run.length)
											.collect::<String>()
											.into(),
										style: Style {
											fg: Some(int_to_colour(style_run.fontColor)),
											..Style::default()
										},
									});

									index = style_run.startIndex + style_run.length;
								}

								lines.push(current_line);
							}
						};
					}

					info.push((
						Paragraph::new(lines).wrap(Wrap { trim: false }),
						Node::Video(self.currentVideoEndpoint.watchEndpoint.videoId.clone()),
					));
				}
				// Video transcript
				EngagementPanelSectionListRenderer::SearchableTranscript { content } => {
					titles.push(ListItem::new("Transcript"));

					info.push((
						Paragraph::new(EMPTY_TEXT),
						Node::Transcript(
							content
								.continuationItemRenderer
								.continuationEndpoint
								.getTranscriptEndpoint
								.params,
						),
					));
				}
				// Comments
				EngagementPanelSectionListRenderer::CommentsSection { header } => {
					// Title
					let mut title =
						Spans::from(header.engagementPanelTitleHeaderRenderer.contextualInfo);
					title.0.push(" Comments".into());
					titles.push(ListItem::new(title));

					info.push((Paragraph::new(EMPTY_TEXT), Node::None));

					for sub_menu_item in header
						.engagementPanelTitleHeaderRenderer
						.menu
						.sortFilterSubMenuRenderer
						.subMenuItems
					{
						titles.push(ListItem::new(sub_menu_item.title));
						info.push((
							Paragraph::new(EMPTY_TEXT),
							Node::CommentSection(
								sub_menu_item.serviceEndpoint.continuationCommand.token,
							),
						));
					}
				}
				// Ignore
				EngagementPanelSectionListRenderer::Ads {} => (),
			}
		}

		// Empty line
		titles.push(ListItem::new(text::Text {
			lines: vec![Spans(Vec::new())],
		}));
		info.push((Paragraph::new(EMPTY_TEXT), Node::None));

		// The 'next up' video

		titles.push(spaced("Autoplay video"));

		let mut lines = vec![
			self.playerOverlays
				.playerOverlayRenderer
				.autoplay
				.playerOverlayAutoplayRenderer
				.videoTitle
				.simpleText
				.into(),
			"".into(),
			self.playerOverlays
				.playerOverlayRenderer
				.autoplay
				.playerOverlayAutoplayRenderer
				.byline
				.into(),
			self.playerOverlays
				.playerOverlayRenderer
				.autoplay
				.playerOverlayAutoplayRenderer
				.publishedTimeText
				.simpleText
				.into(),
			self.playerOverlays
				.playerOverlayRenderer
				.autoplay
				.playerOverlayAutoplayRenderer
				.shortViewCountText
				.accessibility
				.accessibilityData
				.label
				.into(),
		];

		// Video length
		for thumbnail_overlay in self
			.playerOverlays
			.playerOverlayRenderer
			.autoplay
			.playerOverlayAutoplayRenderer
			.thumbnailOverlays
		{
			lines.push(
				thumbnail_overlay
					.thumbnailOverlayTimeStatusRenderer
					.text
					.accessibility
					.accessibilityData
					.label
					.into(),
			);
		}

		info.push((
			Paragraph::new(lines),
			Node::Video(
				self.playerOverlays
					.playerOverlayRenderer
					.autoplay
					.playerOverlayAutoplayRenderer
					.videoId,
			),
		));

		// Reccommendations below the video

		for result in self
			.contents
			.twoColumnWatchNextResults
			.secondaryResults
			.secondaryResults
			.results
		{
			if let Some(continuation_token) = result.into_widgets(&mut titles, &mut info) {
				continuation = Some(continuation_token);
			}
		}

		(titles, info, continuation)
	}
}
