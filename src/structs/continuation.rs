//! Structures returned continuation requests

#![allow(non_snake_case)]

use ratatui::widgets::{ListItem, Paragraph};
use serde::Deserialize;

use super::{ContinueOnResponseReceivedAction, IntoWidgets, Node};

#[derive(Deserialize)]
pub struct ContinuationResponse<T: IntoWidgets> {
	#[serde(alias = "onResponseReceivedEndpoints")]
	onResponseReceivedActions: Vec<ContinueOnResponseReceivedAction<T>>,
	// Ignore `responseContext` and `trackingParams`
}
impl<T: IntoWidgets> IntoWidgets for ContinuationResponse<T> {
	/// Adds items from this response to the existing lists, returning continuation token if it's
	/// given
	fn into_widgets<'a>(
		self,
		list: &mut Vec<ListItem<'a>>,
		info_vec: &mut Vec<(Paragraph<'a>, Node)>,
	) -> Option<String> {
		let mut continuation = None;

		for on_response_received_action in self.onResponseReceivedActions {
			for continuation_item in on_response_received_action
				.appendContinuationItemsAction
				.continuationItems
			{
				if let Some(continuation_token) = continuation_item.into_widgets(list, info_vec) {
					continuation = Some(continuation_token);
				}
			}
		}

		continuation
	}
}
