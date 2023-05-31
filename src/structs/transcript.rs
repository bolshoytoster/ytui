//! Struct returned from transcript requests

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{ListItem, Paragraph};
use serde::Deserialize;

use super::{spaced, underlined, Node, SimpleText, Text, EMPTY_TEXT};

#[derive(Deserialize)]
struct TranscriptSegmentRenderer {
	snippet: Text,
	startTimeText: SimpleText,
	// Ignore `startMs`, `endMs`, `trackingParams`, `accessibility` and `targetId`
}

#[derive(Deserialize)]
struct InitialSegment {
	transcriptSegmentRenderer: TranscriptSegmentRenderer,
}

#[derive(Deserialize)]
struct TranscriptSegmentListRenderer {
	initialSegments: Vec<InitialSegment>,
	// Ignore `noResultLabel`, `retryLabel` and `touchCaptionsEnabled`
}

#[derive(Deserialize)]
struct TranscriptSearchPanelRendererBody {
	transcriptSegmentListRenderer: TranscriptSegmentListRenderer,
}

#[derive(Deserialize)]
struct ReloadContinuationData {
	continuation: String,
	// Ignore `clickTrackingParams`
}

#[derive(Deserialize)]
struct LanguageMenuSortFilterSubMenuRendererSubMenuItemContinuation {
	reloadContinuationData: ReloadContinuationData,
}

#[derive(Deserialize)]
struct LanguageMenuSortFilterSubMenuRendererSubMenuItem {
	title: String,
	selected: bool,
	continuation: LanguageMenuSortFilterSubMenuRendererSubMenuItemContinuation,
}

#[derive(Deserialize)]
struct LanguageMenuSortFilterSubMenuRenderer {
	subMenuItems: Vec<LanguageMenuSortFilterSubMenuRendererSubMenuItem>,
}

#[derive(Deserialize)]
struct LanguageMenu {
	sortFilterSubMenuRenderer: LanguageMenuSortFilterSubMenuRenderer,
}

#[derive(Deserialize)]
struct TranscriptFooterRenderer {
	languageMenu: LanguageMenu,
}

#[derive(Deserialize)]
struct TranscriptSearchPanelRendererFooter {
	transcriptFooterRenderer: TranscriptFooterRenderer,
}

#[derive(Deserialize)]
struct TranscriptSearchPanelRenderer {
	body: TranscriptSearchPanelRendererBody,
	footer: TranscriptSearchPanelRendererFooter, // Ignore `targetId` and `trackingParams`
}

#[derive(Deserialize)]
struct TranscriptRendererContent {
	transcriptSearchPanelRenderer: TranscriptSearchPanelRenderer,
}

#[derive(Deserialize)]
struct TranscriptRenderer {
	content: TranscriptRendererContent, // Ignore   trackingParams`
}

#[derive(Deserialize)]
struct UpdateEngagementPanelActionContent {
	transcriptRenderer: TranscriptRenderer,
}

#[derive(Deserialize)]
struct UpdateEngagementPanelAction {
	content: UpdateEngagementPanelActionContent, // Ignore `targetId`
}

#[derive(Deserialize)]
struct Action {
	updateEngagementPanelAction: UpdateEngagementPanelAction, // Ignore `clickTrackingParams`
}

#[derive(Deserialize)]
pub struct TranscriptResponse {
	actions: Vec<Action>, // Ignore `responseContext` and `trackingParams`
}
impl TranscriptResponse {
	pub fn into_widgets<'a>(self) -> (Vec<ListItem<'a>>, Vec<(Paragraph<'a>, Node)>) {
		// Number of items
		let len = self
			.actions
			.iter()
			.map(|action| {
				action
					.updateEngagementPanelAction
					.content
					.transcriptRenderer
					.content
					.transcriptSearchPanelRenderer
					.footer
					.transcriptFooterRenderer
					.languageMenu
					.sortFilterSubMenuRenderer
					.subMenuItems
					.len() + action
					.updateEngagementPanelAction
					.content
					.transcriptRenderer
					.content
					.transcriptSearchPanelRenderer
					.body
					.transcriptSegmentListRenderer
					.initialSegments
					.len() + 2
			})
			.sum();

		let mut titles = Vec::with_capacity(len);
		let mut info = Vec::with_capacity(len);

		for action in self.actions {
			titles.push(underlined("Other languages"));

			info.push((Paragraph::new(EMPTY_TEXT), Node::None));

			// Other available languages
			for sub_menu_item in action
				.updateEngagementPanelAction
				.content
				.transcriptRenderer
				.content
				.transcriptSearchPanelRenderer
				.footer
				.transcriptFooterRenderer
				.languageMenu
				.sortFilterSubMenuRenderer
				.subMenuItems
			{
				titles.push(ListItem::new(Span {
					content: sub_menu_item.title.into(),
					style: Style {
						add_modifier: if sub_menu_item.selected {
							// Underline this if it's the selected one
							Modifier::UNDERLINED
						} else {
							Modifier::empty()
						},
						..Style::default()
					},
				}));

				info.push((
					Paragraph::new(EMPTY_TEXT),
					Node::Transcript(
						sub_menu_item
							.continuation
							.reloadContinuationData
							.continuation,
					),
				));
			}

			for initial_segment in action
				.updateEngagementPanelAction
				.content
				.transcriptRenderer
				.content
				.transcriptSearchPanelRenderer
				.body
				.transcriptSegmentListRenderer
				.initialSegments
			{
				titles.push(spaced(initial_segment.transcriptSegmentRenderer.snippet));

				info.push((
					Paragraph::new(
						initial_segment
							.transcriptSegmentRenderer
							.startTimeText
							.simpleText,
					),
					Node::None,
				))
			}
		}

		(titles, info)
	}
}
