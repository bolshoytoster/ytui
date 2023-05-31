#![feature(pattern)]
#![feature(option_zip)]
#![feature(exclusive_range_pattern)]

use std::io::{stdin, stdout, Read};
use std::panic::{set_hook, take_hook};

use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use curl::easy;
use curl::easy::Easy;
use js_sandbox::Script;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use simd_json::{from_slice, Error, ErrorType};

mod config;
use config::*;
mod structs;
use structs::*;
mod utils;
use utils::*;

/// Current page + information on previous pages
pub enum Page {
	Home {
		continuation: Option<String>,
	},
	Category {
		continuation: Option<String>,
		/// Previous page, needs to be on heap to avoid recursive type. The second field is the
		/// index to the selected item
		previous: (Box<Page>, usize),
	},
	Game {
		browse_id: String,
		continuation: Option<String>,
		params: Option<String>,
		previous: (Box<Page>, usize),
	},
	/// Search results
	Search {
		query: String,
		params: Option<String>,
		continuation: Option<String>,
		previous: (Box<Page>, usize),
	},
	/// Recommendations
	Next {
		video_id: String,
		continuation: Option<String>,
		previous: (Box<Page>, usize),
	},
	Transcript {
		params: String,
		previous: (Box<Page>, usize),
	},
	/// Comment section
	CommentSection {
		/// Token for the first page of comments
		first_continuation: String,
		/// Token for subsequent comments
		continuation: Option<String>,
		previous: (Box<Page>, usize),
	},
	/// A comment's replies
	Comment {
		first_continuation: String,
		continuation: Option<String>,
		previous: (Box<Page>, usize),
	},
}
impl Page {
	fn request<'a>(&mut self, easy: &mut Easy) -> (Vec<ListItem<'a>>, Vec<(Paragraph<'a>, Node)>) {
		match self {
			Page::Home {
				ref mut continuation,
				..
			} => {
				// Search for the JSON in a <script> tag in HTML (just the bit that's needed)
				// (between `{"content` and `1}}`) (I know this is terrible, there'd still need to
				// be something like this even with a HTML parser, since it's embedded in js).
				let (list, info, new_continuation) = extract_json::<RichGridRenderer>(
					&mut unsafe {
						String::from_utf8_unchecked(request_get(easy, "https://www.youtube.com"))
					},
					"{\"cont",
					"1}}",
					0,
				)
				.expect("youtube's home page js format may have changed")
				.into_widgets();

				*continuation = new_continuation;

				(list, info)
			}
			Page::Category { continuation, .. } => {
				let request = BrowseRequest {
					continuation: continuation.clone(),
					..BrowseRequest::new(easy).expect("Youtube should set `__Secure-YEC` cookie")
				};

				let mut list = Vec::new();
				let mut info_vec = Vec::new();

				from_slice::<ContinuationResponse<RichGridRendererContent>>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/browse",
					&request,
				))
				.expect("Category JSON should be valid")
				.into_widgets(&mut list, &mut info_vec);

				(list, info_vec)
			}
			Page::Game {
				browse_id,
				ref mut continuation,
				params,
				..
			} => {
				let request = BrowseRequest {
					browseId: Some(browse_id.clone()),
					params: params.clone(),
					..BrowseRequest::new(easy).expect("Youtube should set `__Secure-YEC` cookie")
				};

				let (list, info, new_continuation) = from_slice::<GeneralResponse>(
					&mut request_post(easy, "https://www.youtube.com/youtubei/v1/browse", &request),
				)
				.expect("Game JSON should be valid")
				.into_widgets();

				*continuation = new_continuation;

				(list, info)
			}
			Page::Search {
				query,
				params,
				ref mut continuation,
				..
			} => {
				let (list, info, new_continuation) =
					from_slice::<SearchResponse>(&mut request_post(
						easy,
						"https://www.youtube.com/youtubei/v1/search",
						&SearchRequest {
							query: query.clone(),
							params: params.clone(),
							..SearchRequest::default()
						},
					))
					.expect("Next JSON should be valid")
					.into_widgets();

				*continuation = new_continuation;

				(list, info)
			}
			Page::Next {
				video_id,
				ref mut continuation,
				..
			} => {
				let (list, info, new_continuation) = from_slice::<NextResponse>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/next",
					&NextRequest {
						videoId: video_id.clone(),
						..NextRequest::default()
					},
				))
				.expect("Next JSON should be valid")
				.into_widgets();

				*continuation = new_continuation;

				(list, info)
			}
			Page::Transcript { params, .. } => from_slice::<TranscriptResponse>(&mut request_post(
				easy,
				"https://www.youtube.com/youtubei/v1/get_transcript",
				&BrowseRequest {
					params: Some(params.clone()),
					..BrowseRequest::default()
				},
			))
			.expect("Transcript response should be valid")
			.into_widgets(),
			Page::CommentSection {
				first_continuation,
				ref mut continuation,
				..
			} => {
				let mut list = Vec::new();
				let mut info_vec = Vec::new();

				*continuation = from_slice::<CommentsResponse>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/next",
					&BrowseRequest {
						continuation: Some(first_continuation.clone()),
						..BrowseRequest::default()
					},
				))
				.expect("Comments JSON should be valid")
				.into_widgets(&mut list, &mut info_vec);

				(list, info_vec)
			}
			Page::Comment {
				first_continuation,
				ref mut continuation,
				..
			} => {
				let mut list = Vec::new();
				let mut info = Vec::new();

				*continuation = from_slice::<ContinuationResponse<Comment>>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/next",
					&BrowseRequest {
						continuation: Some(first_continuation.clone()),
						..BrowseRequest::default()
					},
				))
				.expect("Next JSON should be valid")
				.into_widgets(&mut list, &mut info);

				(list, info)
			}
		}
	}

	/// Continue this page, adds items to the passed `Vec`s
	fn r#continue<'a>(
		&mut self,
		easy: &mut Easy,
		list: &mut Vec<ListItem<'a>>,
		info_vec: &mut Vec<(Paragraph<'a>, Node)>,
	) {
		match self {
			Page::Home {
				continuation: continuation @ Some(_),
				..
			} => {
				let request = BrowseRequest {
					continuation: continuation.take(),
					..BrowseRequest::new(easy).expect("Youtube should set `__Secure-YEC` cookie")
				};

				*continuation = from_slice::<ContinuationResponse<RichGridRendererContent>>(
					&mut request_post(easy, "https://www.youtube.com/youtubei/v1/browse", &request),
				)
				.expect("Continuation JSON should be valid")
				.into_widgets(list, info_vec);
			}
			Page::Game {
				continuation: continuation @ Some(_),
				..
			} => {
				let request = BrowseRequest {
					continuation: continuation.take(),
					..BrowseRequest::new(easy).expect("Youtube should set `__Secure-YEC` cookie")
				};

				*continuation = from_slice::<ContinuationResponse<RichGridRendererContent>>(
					&mut request_post(easy, "https://www.youtube.com/youtubei/v1/browse", &request),
				)
				.expect("Continuation JSON should be valid")
				.into_widgets(list, info_vec);
			}
			Page::Search {
				continuation: continuation @ Some(_),
				..
			} => {
				*continuation = from_slice::<SearchContinuationResponse>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/search",
					&BrowseRequest {
						continuation: continuation.take(),
						..BrowseRequest::default()
					},
				))
				.expect("Continuation JSON should be valid")
				.into_widgets(list, info_vec);
			}
			Page::Next {
				continuation: continuation @ Some(_),
				..
			} => {
				let request = BrowseRequest {
					continuation: continuation.take(),
					..BrowseRequest::new(easy).expect("Youtube should set `__Secure-YEC` cookie")
				};

				*continuation = from_slice::<ContinuationResponse<SecondaryResultsResult>>(
					&mut request_post(easy, "https://www.youtube.com/youtubei/v1/next", &request),
				)
				.expect("Continuation JSON should be valid")
				.into_widgets(list, info_vec);
			}
			Page::CommentSection {
				continuation: continuation @ Some(_),
				..
			} => {
				*continuation =
					from_slice::<ContinuationResponse<ContinuationItem>>(&mut request_post(
						easy,
						"https://www.youtube.com/youtubei/v1/next",
						&BrowseRequest {
							continuation: continuation.take(),
							..BrowseRequest::default()
						},
					))
					.expect("Comments JSON should be valid")
					.into_widgets(list, info_vec);
			}
			Page::Comment {
				continuation: continuation @ Some(_),
				..
			} => {
				*continuation = from_slice::<ContinuationResponse<Comment>>(&mut request_post(
					easy,
					"https://www.youtube.com/youtubei/v1/next",
					&BrowseRequest {
						continuation: continuation.take(),
						..BrowseRequest::default()
					},
				))
				.expect("Next JSON should be valid")
				.into_widgets(list, info_vec)
			}
			// No continuation token or can't be continued
			_ => (),
		}
	}
}
impl ToString for Page {
	/// Get this page's title
	fn to_string(&self) -> String {
		match self {
			Page::Home { .. } => "Home",
			Page::Category { .. } => "A category",
			Page::Game { .. } => "A game",
			Page::Search { query, .. } => query,
			Page::Next { .. } => "Recommendations",
			Page::Transcript { .. } => "Transcript",
			Page::CommentSection { .. } => "Comments",
			Page::Comment { .. } => "A comment",
		}
		.to_owned()
	}
}

fn main() {
	let mut easy = Easy::new();

	// Enable cookie engine
	let _ = easy.cookie_file("");

	let mut easy_list = easy::List::new();
	// Youtube needs the header for post requests
	let _ = easy_list.append("CONTENT-TYPE:");
	let _ = easy.http_headers(easy_list);

	let hook = take_hook();
	// Run cleanup code on panic
	set_hook(Box::new(move |panic_info| {
		let _ = disable_raw_mode();
		let _ = execute!(stdout(), LeaveAlternateScreen);
		hook(panic_info);
	}));

	let mut page = Page::Home { continuation: None };

	// Cached JS script (it's used to decrypt stuff to avoid throttling)
	let mut js_script = None;

	// Fetch data
	let (mut list, mut info_vec) = page.request(&mut easy);

	let mut ratatui_list = List::new(list.clone()).highlight_style(Style {
		add_modifier: Modifier::REVERSED,
		..Style::default()
	});

	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
		.expect("Should be able to initialize terminal");

	// Init crossterm
	let _ = enable_raw_mode();

	let _ = execute!(stdout(), EnterAlternateScreen);

	// Clear screen
	let _ = terminal.clear();

	// Should we redraw this frame?
	let mut redraw = true;

	let mut list_state = ListState::default();
	list_state.select(Some(0));

	loop {
		if redraw {
			let _ = terminal.draw(|frame| {
				// Left panel border
				frame.render_widget(
					Block::default()
						.title(page.to_string())
						.borders(Borders::ALL)
						.title_alignment(TITLE_ALIGNMENT)
						.border_type(BORDER_TYPE),
					Rect {
						width: frame.size().width / 2,
						..frame.size()
					},
				);
				// Left panel list
				frame.render_stateful_widget_reusable(
					&ratatui_list,
					Rect {
						x: 2,
						y: 2,
						width: frame.size().width / 2 - 4,
						height: frame.size().height - 3,
					},
					&mut list_state,
				);

				// Right panel border
				frame.render_widget(
					Block::default()
						.borders(Borders::ALL)
						.title_alignment(TITLE_ALIGNMENT)
						.border_type(BORDER_TYPE),
					Rect {
						x: frame.size().width / 2,
						width: (frame.size().width + 1) / 2,
						..frame.size()
					},
				);
				// Top-right panel text
				frame.render_widget_reusable(
					&info_vec[list_state.selected().expect("Something should be selected")].0,
					Rect {
						x: frame.size().width / 2 + 2,
						y: 2,
						width: (frame.size().width - 7) / 2,
						height: frame.size().height - 4,
					},
				);

				// Bottom-right panel text
				frame.render_widget(
					Paragraph::new(vec![
						"back: b".into(),
						"search: s".into(),
						"refresh: r".into(),
						"next: n".into(),
						"quit: q".into(),
					])
					.alignment(Alignment::Right),
					Rect {
						x: frame.size().width / 2 + 2,
						y: frame.size().height - 7,
						width: (frame.size().width - 7) / 2,
						height: 7,
					},
				);
			});
		}

		redraw = true;

		// Read input
		match read().expect("IO error") {
			Event::Key(KeyEvent { code, .. }) => match code {
				// Quit
				KeyCode::Char('Q' | 'q') => break,
				// Move down
				KeyCode::Down | KeyCode::Char('J' | 'j') => {
					list_state.select(list_state.selected().map(|s| {
						// Load next page if we're at the bottom
						if s + 1 == info_vec.len() {
							page.r#continue(&mut easy, &mut list, &mut info_vec);

							ratatui_list = List::new(list.clone()).highlight_style(Style {
								add_modifier: Modifier::REVERSED,
								..Style::default()
							});
						}

						info_vec.len().min(s + 2) - 1
					}))
				}
				// Move up
				KeyCode::Up | KeyCode::Char('K' | 'k') => {
					list_state.select(list_state.selected().map(|s| s.saturating_sub(1)))
				}
				KeyCode::PageDown => list_state.select(list_state.selected().map(|s| {
					let new_position = s
						+ (terminal
							.size()
							.expect("Should be able to get terminal height")
							.height / 2) as usize;

					// Load next page if we've gone past the end
					if info_vec.len() <= new_position + 1 {
						page.r#continue(&mut easy, &mut list, &mut info_vec);

						ratatui_list = List::new(list.clone()).highlight_style(Style {
							add_modifier: Modifier::REVERSED,
							..Style::default()
						});
					}

					info_vec.len().min(new_position + 2) - 1
				})),
				KeyCode::PageUp => list_state.select(list_state.selected().map(|s| {
					s.saturating_sub(
						(terminal
							.size()
							.expect("Should be able to get terminal height")
							.height / 2 - 1) as usize,
					)
				})),
				KeyCode::Right | KeyCode::Char('L' | 'l') => {
					// Enter
					if match &info_vec[list_state.selected().expect("Something should be selected")]
						.1
					{
						Node::Header(continuation) => {
							// Category
							page = Page::Category {
								continuation: Some(continuation.clone()),
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							// Reload page
							true
						}
						Node::Video(video_id) => {
							// Video
							if let Some(ref mut script) = js_script {
								// If JS has already been initialized, we can directly get the video
								// data and play it
								match from_slice::<VideoResponse>(&mut request_post(
									&mut easy,
									"https://www.youtube.com/youtubei/v1/player",
									&BrowseRequest {
										videoId: Some(video_id.clone()),
										..BrowseRequest::default()
									},
								)) {
									Ok(parsed_response) => parsed_response.play(script),
									// Common error for unavailable videos, don't panic
									Err(error)
										if error
											== Error::generic(ErrorType::Serde(
												"missing field `streamingData`".to_owned(),
											)) =>
									{
										// Temporarily move into normal terminal
										let _ = disable_raw_mode();
										let _ = execute!(stdout(), LeaveAlternateScreen);

										print!(
											"Couldn't parse response, it may be age restricted \
											 (press enter to continue) "
										);

										// Wait for enter
										let _ = stdin().read(&mut [0]);

										let _ = enable_raw_mode();
										let _ = execute!(stdout(), EnterAlternateScreen);
									}
									// Panic on any other errors
									Err(error) => panic!("Failed to parse video response: {error}"),
								}
							} else {
								// We need to initialize js context first, download the whole page
								let mut response = unsafe {
									String::from_utf8_unchecked(request_get(
										&mut easy,
										&["https://www.youtube.com/watch/", video_id].concat(),
									))
								};

								// Find the current name of the player JS
								let start = response
									.find("c=\"/")
									.expect("Video page should have the /s/player/.../base.js")
									+ 3;

								// Download the player
								let mut player = unsafe {
									String::from_utf8_unchecked(request_get(
										&mut easy,
										&[
											"https://www.youtube.com",
											&response[start
												..start
													+ response[start..].find('"').expect(
														"Video page should have the \
														 /s/player/.../base.js",
													)],
										]
										.concat(),
									))
								};

								// Find the current n decryption function in the player js and
								// define it in our context
								let n_start = player
									.find("n(a){var b=a.sp")
									.expect("Should be an `n` decryption function")
									- 9;

								// Use a determined function name ('f')
								player.replace_range(n_start..n_start + 1, "f");

								// Find the signature decipher function
								let sig_start = player
									.find("a=a.split(\"\"")
									.expect("Should be a `sig` decryption function")
									- 14;

								// Use a determined function name ('s')
								player.replace_range(sig_start..sig_start + 1, "s");

								// Create new script and define `n` function in it
								let mut script = Script::from_string(
									&[
										// `n` function
										&player[n_start
											..n_start
												+ player[n_start..]
													.find("\ng")
													.expect("`n` function should end with this")],
										// I don't think this changes. (between 'VF=' and '};g.W')
										"VF={RV:function(a,b){var \
										 c=a[0];a[0]=a[b%a.length];a[b%a.length]=c},p4:function(a,\
										 b){a.splice(0,b)},wa:function(a){a.reverse()}};",
										// `sig` function
										&player[sig_start
											..sig_start
												+ player[sig_start..]
													.find(";\n")
													.expect("`sig` function should end with this")],
									]
									.concat(),
								)
								.expect("`n` function should be valid");

								// Extract the JSON data
								if let Some(parsed_response) =
									extract_json::<VideoResponse>(&mut response, "{\"re", "};", 1)
								{
									parsed_response.play(&mut script);
								} else {
									println!("Couldn't parse response, it may be age restricted");
								}

								js_script = Some(script);
							}

							// Don't reload page
							false
						}
						Node::Game(browse_id, params) => {
							page = Page::Game {
								browse_id: browse_id.clone(),
								continuation: None,
								params: params.clone(),
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							true
						}
						Node::Search(query, params) => {
							page = Page::Search {
								query: query.clone(),
								params: params.clone(),
								continuation: None,
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							true
						}
						Node::Channel(browse_id, params) => todo!(),
						Node::Playlist(playlist_id) => todo!(),
						Node::Transcript(params) => {
							page = Page::Transcript {
								params: params.clone(),
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							true
						}
						Node::CommentSection(first_continuation) => {
							page = Page::CommentSection {
								first_continuation: first_continuation.clone(),
								continuation: None,
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							true
						}
						Node::Comment(first_continuation) => {
							page = Page::Comment {
								first_continuation: first_continuation.clone(),
								continuation: None,
								previous: (
									Box::new(page),
									list_state.selected().expect("Something should be selected"),
								),
							};

							true
						}
						// This can't be selected, do nothing
						Node::None => false,
					} {
						// If we selected a category
						// Move cursor to the top
						list_state.select(Some(0));

						(list, info_vec) = page.request(&mut easy);
						ratatui_list = List::new(list.clone()).highlight_style(Style {
							add_modifier: Modifier::REVERSED,
							..Style::default()
						});
					}

					let _ = terminal.clear();
				}
				// Go back
				KeyCode::Left | KeyCode::Char('B' | 'b') => {
					match page {
						// Just move cursor to the top
						Page::Home { .. } => list_state.select(Some(0)),
						Page::Category { previous, .. }
						| Page::Game { previous, .. }
						| Page::Search { previous, .. }
						| Page::Next { previous, .. }
						| Page::Transcript { previous, .. }
						| Page::CommentSection { previous, .. }
						| Page::Comment { previous, .. } => {
							page = *previous.0;
							list_state.select(Some(previous.1.min(info_vec.len() - 1)));

							(list, info_vec) = page.request(&mut easy);
							ratatui_list = List::new(list.clone()).highlight_style(Style {
								add_modifier: Modifier::REVERSED,
								..Style::default()
							});
						}
					}
				}
				// Home
				KeyCode::Char('H' | 'h') => {
					// Move cursor to the top
					list_state.select(Some(0));

					page = Page::Home { continuation: None };
					(list, info_vec) = page.request(&mut easy);
					ratatui_list = List::new(list.clone()).highlight_style(Style {
						add_modifier: Modifier::REVERSED,
						..Style::default()
					});
				}
				// Search
				KeyCode::Char('S' | 's' | '/') => {
					// Show cursor
					let _ = terminal.show_cursor();

					let mut query = String::new();

					loop {
						let _ = terminal.draw(|frame| {
							// Width of the input box
							let width = (query.len() as u16 + 3).clamp(8, frame.size().width);

							frame.render_widget(
								Paragraph::new(query.clone()).block(
									Block::default()
										.borders(Borders::ALL)
										.title("Search")
										.title_alignment(TITLE_ALIGNMENT)
										.border_type(BORDER_TYPE),
								),
								Rect {
									x: (frame.size().width - width) / 2,
									y: frame.size().height / 2 - 1,
									width,
									height: 3,
								},
							)
						});

						if let Event::Key(KeyEvent { code, .. }) =
							read().expect("Should be able to read input")
						{
							match code {
								KeyCode::Char(c) => query.push(c),
								KeyCode::Backspace => {
									query.pop();
								}
								KeyCode::Enter => break,
								_ => (),
							}
						}
					}

					page = Page::Search {
						query,
						params: None,
						continuation: None,
						previous: (
							Box::new(page),
							list_state.selected().expect("Something should be selected"),
						),
					};

					// Move cursor to the top
					list_state.select(Some(0));

					(list, info_vec) = page.request(&mut easy);

					ratatui_list = List::new(list.clone()).highlight_style(Style {
						add_modifier: Modifier::REVERSED,
						..Style::default()
					});

					let _ = terminal.clear();

					// Hide the cursor again
					let _ = terminal.hide_cursor();
				}
				// Refresh
				KeyCode::Char('R' | 'r') => {
					// Just send this page's request again and parse it
					(list, info_vec) = page.request(&mut easy);
					ratatui_list = List::new(list.clone()).highlight_style(Style {
						add_modifier: Modifier::REVERSED,
						..Style::default()
					});

					// Make sure the cursor isn't past the end of the data
					list_state.select(list_state.selected().map(|s| s.min(info_vec.len() - 1)));
				}
				KeyCode::Char('N' | 'n') => {
					if let Node::Video(video_id) =
						&info_vec[list_state.selected().expect("Something should be selected")].1
					{
						page = Page::Next {
							video_id: video_id.clone(),
							continuation: None,
							previous: (
								Box::new(page),
								list_state.selected().expect("Something should be selected"),
							),
						};

						// Move cursor to the top
						list_state.select(Some(0));

						(list, info_vec) = page.request(&mut easy);
						ratatui_list = List::new(list.clone()).highlight_style(Style {
							add_modifier: Modifier::REVERSED,
							..Style::default()
						});
					}
				}
				_ => redraw = false,
			},
			// We want to redraw
			Event::Resize(..) => (),
			_ => redraw = false,
		}
	}

	let _ = disable_raw_mode();
	let _ = execute!(stdout(), LeaveAlternateScreen);
}
