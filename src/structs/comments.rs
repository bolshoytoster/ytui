//! Structures returned from a comments response.

use ratatui::style::Style;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{ListItem, Paragraph, Wrap};
use serde::Deserialize;

use super::{
	int_to_colour, spaced, AccessibleText, Color, ContinuationEndpoint, ContinuationItemRenderer,
	Endpoint, IntoWidgets, Node, SimpleText, Text,
};

#[derive(Deserialize)]
struct PinnedCommentBadgeRenderer {
	label: Text,
	color: Color, // Ignore `icon`
}

#[derive(Deserialize)]
struct PinnedCommentBadge {
	pinnedCommentBadgeRenderer: PinnedCommentBadgeRenderer,
}

#[derive(Deserialize)]
struct AuthorCommentBadgeRenderer {
	color: Option<Color>,
	iconTooltip: String,
	// Ignore `icon`, `authorText` and `authorEndpoint`
}

#[derive(Deserialize)]
struct AuthorCommentBadge {
	authorCommentBadgeRenderer: AuthorCommentBadgeRenderer,
}

#[derive(Deserialize)]
pub struct CommentRenderer {
	authorText: SimpleText,
	authorEndpoint: Endpoint,
	contentText: Text,
	publishedTimeText: Text,
	authorIsChannelOwner: bool,
	voteCount: Option<AccessibleText>,
	pinnedCommentBadge: Option<PinnedCommentBadge>,
	authorCommentBadge: Option<AuthorCommentBadge>,
	/// Max is 501, so fits in u16
	replyCount: Option<u16>,
	// Ignore `authorThumbnails`, `isLiked`, `commentId`, `actionButtons`, `voteStatus`,
	// `trackingParams`, `expandButton`, `collapseButton` and `loggingDirective`
}

#[derive(Deserialize)]
struct ButtonRenderer {
	command: ContinuationEndpoint, // Ignore `text`, `icon` and `trackingParams`
}

#[derive(Deserialize)]
struct Button {
	buttonRenderer: ButtonRenderer,
}

#[derive(Deserialize)]
pub struct CommentContinuationItemRenderer {
	button: Button,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Comment {
	Comment {
		/// Boxed to greatly reduce enum's size (and to stop clippy warning)
		commentRenderer: Box<CommentRenderer>,
	},
	/// Should only appear for replies
	ContinuationItem {
		continuationItemRenderer: CommentContinuationItemRenderer,
	},
}
impl IntoWidgets for Comment {
	/// Should only return a continuation token for replies
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			Comment::Comment { commentRenderer } => {
				titles.push(spaced(commentRenderer.authorText.simpleText));

				let mut lines = vec![
					commentRenderer.contentText.into(),
					"".into(),
					commentRenderer.publishedTimeText.into(),
				];

				// Comments that haven't been liked/disliked yet don't have a like count
				if let Some(vote_count) = commentRenderer.voteCount {
					lines.push(vote_count.accessibility.accessibilityData.label.into());
				}

				// If this commenter is the video uploader
				if commentRenderer.authorIsChannelOwner {
					lines.push("Video uploader".into());
				}

				// For pinned comments
				if let Some(pinned_comment_badge) = commentRenderer.pinnedCommentBadge {
					lines.push(
						pinned_comment_badge
							.pinnedCommentBadgeRenderer
							.label
							.with_style(Style {
								fg: Some(int_to_colour(
									pinned_comment_badge
										.pinnedCommentBadgeRenderer
										.color
										.basicColorPaletteData
										.foregroundTitleColor,
								)),
								..Style::default()
							}),
					);
				}

				// Commenter badge
				if let Some(author_comment_badge) = commentRenderer.authorCommentBadge {
					lines.push(
						if let Some(colour) = author_comment_badge.authorCommentBadgeRenderer.color
						{
							Spans(vec![Span {
								content: author_comment_badge
									.authorCommentBadgeRenderer
									.iconTooltip
									.into(),
								style: Style {
									fg: Some(int_to_colour(
										colour.basicColorPaletteData.foregroundTitleColor,
									)),
									bg: colour
										.basicColorPaletteData
										.backgroundColor
										.map(int_to_colour),
									..Style::default()
								},
							}])
						} else {
							author_comment_badge
								.authorCommentBadgeRenderer
								.iconTooltip
								.into()
						},
					);
				}

				// Number of replies
				if let Some(reply_count) = commentRenderer.replyCount {
					lines.push([&reply_count.to_string(), " replies"].concat().into());
				}

				info.push((Paragraph::new(lines).wrap(Wrap { trim: false }), Node::None));

				None
			}
			Comment::ContinuationItem {
				continuationItemRenderer,
			} => Some(
				continuationItemRenderer
					.button
					.buttonRenderer
					.command
					.continuationCommand
					.token,
			),
		}
	}
}

#[derive(Deserialize)]
struct Content {
	continuationItemRenderer: ContinuationItemRenderer,
}

#[derive(Deserialize)]
struct CommentRepliesRenderer {
	contents: Vec<Content>,
	// Ignore `viewReplies`, `trackingParams` and `targetId`
}

#[derive(Deserialize)]
struct Replies {
	commentRepliesRenderer: CommentRepliesRenderer,
}

#[derive(Deserialize)]
pub struct CommentThreadRenderer {
	comment: Comment,
	replies: Option<Replies>,
	// Ignore `isModeratedElqComment`, `loggingDirectives`, `renderingPriority` and
	// `trackingParams`
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ContinuationItem {
	CommentThread {
		/// Boxed to greatly reduce enum size (and to stop clippy warning)
		commentThreadRenderer: Box<CommentThreadRenderer>,
	},
	#[allow(clippy::enum_variant_names)]
	ContinuationItem {
		continuationItemRenderer: ContinuationItemRenderer,
	},
	/// Ignore the header
	CommentsHeader {},
}
impl IntoWidgets for ContinuationItem {
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		match self {
			ContinuationItem::CommentThread {
				commentThreadRenderer,
			} => {
				// Ignore continuation token, there shouldn't be one
				commentThreadRenderer.comment.into_widgets(titles, info);

				// Set the link of the just added comment to link to the replies, if there are any.
				if let Some(mut replies) = commentThreadRenderer.replies {
					info.last_mut().expect("An item was just added").1 = Node::Comment(
						replies
							.commentRepliesRenderer
							.contents
							.swap_remove(0)
							.continuationItemRenderer
							.continuationEndpoint
							.continuationCommand
							.token,
					);
				}
			}
			ContinuationItem::ContinuationItem {
				continuationItemRenderer,
			} => {
				return Some(
					continuationItemRenderer
						.continuationEndpoint
						.continuationCommand
						.token,
				);
			}
			// Ignore the header
			ContinuationItem::CommentsHeader {} => (),
		}

		None
	}
}

#[derive(Deserialize)]
struct ReloadContinuationItemsCommand {
	continuationItems: Vec<ContinuationItem>, // Ignore `slot` and `targetId`
}

#[derive(Deserialize)]
struct OnResponseReceivedEndpoint {
	reloadContinuationItemsCommand: ReloadContinuationItemsCommand, // Ignore `clickTrackingParams`
}

/// The response from a comments request.
#[derive(Deserialize)]
pub struct CommentsResponse {
	onResponseReceivedEndpoints: Vec<OnResponseReceivedEndpoint>,
	// Ignore `responseContext` and `trackingParams`
}
impl IntoWidgets for CommentsResponse {
	fn into_widgets<'a>(
		self,
		titles: &mut Vec<ListItem<'a>>,
		info: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		// Continuation token
		let mut continuation = None;

		for on_received_endpoint in self.onResponseReceivedEndpoints {
			for continuation_item in on_received_endpoint
				.reloadContinuationItemsCommand
				.continuationItems
			{
				if let Some(continuation_token) = continuation_item.into_widgets(titles, info) {
					continuation = Some(continuation_token);
				}
			}
		}

		continuation
	}
}
