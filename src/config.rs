#![allow(dead_code)]
#![allow(clippy::derivable_impls)]

use ratatui::layout::Alignment;
use ratatui::widgets::BorderType;

use crate::structs::*;

pub enum Selector<T> {
	Highest,
	ClosestTo(T),
	Lowest,
}
impl<T: std::ops::Sub<T> + Ord + Clone> Selector<T>
where
	<T as std::ops::Sub>::Output: Ord,
{
	/// Returns true if y is better than y, according to this selector.
	pub fn is_better(&self, x: T, y: T) -> std::cmp::Ordering {
		match &self {
			Self::Highest => y.cmp(&x),
			Selector::ClosestTo(target) => {
				fn abs_diff<T: std::ops::Sub<T> + Ord>(a: T, b: T) -> <T as std::ops::Sub>::Output {
					if a < b { b - a } else { a - b }
				}
				abs_diff(target.clone(), x).cmp(&abs_diff(target.clone(), y))
			}
			Self::Lowest => x.cmp(&y),
		}
	}
}

pub enum MediaFormat {
	Mp4,
	Webm,
}
impl ToString for MediaFormat {
	fn to_string(&self) -> String {
		match self {
			MediaFormat::Mp4 => "mp4",
			MediaFormat::Webm => "webm",
		}
		.to_owned()
	}
}

pub enum VideoSelector {
	Bitrate(Selector<u32>),
	Quality(Selector<u16>),
	Format(MediaFormat),
}

/// Specifies how to pick video quality, in order of priority.
/// If you don't specify any, the first (highest quality) format is used.
/// You can choose any of:
/// `VideoSelector::Bitrate(Selector)`
/// `VideoSelector::Quality(Selector)`
/// `VideoSelector::Format(MediaFormat::Webm/Mp4)`
/// Where `Selector` is one of:
/// `Selector::Highest`
/// `Selector::ClosestTo(number)`
/// `Selector::Lowest`
///
/// The default is an example to show 720p videos at the lowest bitrate available
pub const VIDEO_SELECTOR: &[VideoSelector] = &[
	VideoSelector::Quality(Selector::ClosestTo(720)),
	VideoSelector::Bitrate(Selector::Lowest),
];

pub enum AudioSelector {
	Bitrate(Selector<u32>),
	Language(&'static str),
	Format(MediaFormat),
}

/// Specifies how to pick audio quality, in order of priority.
/// If you don't specify any, the first (highest quality) format is used.
/// See `VIDEO_SELECTOR` above for info on `AudioSelector::BitrateFormat` or `Selector`.
/// You can also use `AudioSelector::Language("Gibberish")` to specify an audio language (not
/// subtitles).
pub const AUDIO_SELECTOR: &[AudioSelector] = &[
	AudioSelector::Language("English"),
	// Lowest bitrate audio by default because most people can't tell the difference
	AudioSelector::Bitrate(Selector::Lowest),
];

/// Defines the caption track to pass to the `video_player` function below. Should either be `None`
/// (no captions) or `Some("Language")`, where "Language" is case sensitive.
pub const CAPTION_LANGUAGE: Option<&'static str> = None;

/// The video player to use. This function should return a tuple, the first item being the command
/// to run, the second item is an iterable of arguments. I decided to make this a function to allow
/// you to use any program, including ones with potentially 'unorthadox' argument passing.
/// The default is an example for mpv, not using the subtitles.
#[allow(unused_variables)]
pub fn video_player(
	video_url: String,
	audio_url: String,
	// `None` if no subtitle language is set (above) or if there are no matching subtitles
	subtitle_url: Option<String>,
) -> (
	impl AsRef<std::ffi::OsStr>,
	impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) {
	("mpv", [format!("--audio-file={audio_url}"), video_url])
}

/// The command used to play streams, similar to `video_player` above. `hls_manifest_url` is the
/// url to the stream (it's m3u8 format, most players should support it)
pub fn stream_player(
	hls_manifest_url: String,
) -> (
	impl AsRef<std::ffi::OsStr>,
	impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) {
	// mpv is rubbish at playing streams, so I'm using ffplay (it's usually installed with ffmpeg,
	// which is installed along with mpv, so it should be available)
	// ffplay will pick the first stream by default (the lowest quality), you can switch to better
	// ones with `v`.
	("ffplay", [hls_manifest_url])
}

// ----------------
// The following settings are for the program's style.
// ----------------

/// Where the title is at the top of the screen.
/// Can be `Left`, `Center` or `Right`.
pub const TITLE_ALIGNMENT: Alignment = Alignment::Left;

/// The style of the UI's borders.
/// Can be `Plain`, `Thick`, `Double` or `Rounded`.
pub const BORDER_TYPE: BorderType = BorderType::Plain;

// ----------------
// The following settings are for API request options, changing some of these could cause the
// server to return errors, which may cause this program to panic. Edit them at your own risk.
// Most of the values are optional, so are `None` by default. Unless stated otherwise, they're
// probably strings, so you could change them to `Some("foo")` if you want to use them.
// ----------------

// These 9 are for tags on the home page

impl Default for ConfigInfo {
	fn default() -> Self {
		// If you want to use this, you need to set `configInfo: Some(ConfigInfo::default())` in
		// `Client`
		Self {
			// This could be a string, i.e. `Some("foo")`
			appInstallData: None,
		}
	}
}

impl Default for MainAppWebInfo {
	fn default() -> Self {
		// If you want to use this, you need to set `mainAppWebInfo:
		// Some(MainAppWebInfo::default())` in `Client`
		Self {
			graftUrl: None,
			webDisplayMode: None,
			// Bool, could be `Some(true)`
			isWebNativeShareAvailable: None,
		}
	}
}

impl Default for Client {
	fn default() -> Self {
		Self {
			hl: None,
			gl: None,
			remoteHost: None,
			deviceMake: None,
			deviceModel: None,
			visitorData: None,
			userAgent: None,
			clientName: "WEB",
			// This may need to be updated at some point
			clientVersion: "2.0000011",
			osName: None,
			osVersion: None,
			originalUrl: None,
			platform: None,
			clientFormFactor: None,
			// Could be `Some(ConfigInfo::default())`
			configInfo: None,
			browserName: None,
			browserVersion: None,
			acceptHeader: None,
			deviceExperimentId: None,
			// The next 5 are integers, could be `Some(0)`
			screenWidthPoints: None,
			screenHeightPoints: None,
			screenPixelDensity: None,
			screenDensityFloat: None,
			utcOffsetMinutes: None,
			userInterfaceTheme: None,
			// Could be `Some(MainAppWebInfo::default())`
			mainAppWebInfo: None,
			timeZone: None,
		}
	}
}

impl Default for User {
	fn default() -> Self {
		// If you want to use this, you need to set `user: Some(User::default())` in `Context`
		Self {
			// Bool, could be `Some(true)`
			lockedSafetyMode: None,
		}
	}
}

impl Default for Request {
	fn default() -> Self {
		// If you want to use this, you need to set `request: Some(Request::default())` in `Context`
		Self {
			// Bool, could be `Some(true)`
			useSsl: None,
			// I don't know what these are so you can't really use them
			internalExperimentFlags: None,
			consistencyTokenJars: None,
		}
	}
}

impl Default for AdSignalsInfo {
	fn default() -> Self {
		// If you want to use this, you need to set `adSignalsInfo: Some(AdSignalsInfo::default())`
		// in `Context`
		Self {
			// Vec<Param>, could be:
			// `Some(vec![Param { key: "foo", value: "bar" }])`
			params: None,
		}
	}
}

impl Default for Context {
	fn default() -> Self {
		Self {
			client: Client::default(),
			// Could be `Some(User::default())`
			user: None,
			// Could be `Some(Request::default())`
			request: None,
			// Could be `Some(AdSignalsInfo::default())`
			adSignalsInfo: None,
		}
	}
}

impl Default for BrowseRequest {
	fn default() -> Self {
		Self {
			context: Context::default(),
			// These four are set by the program
			continuation: None,
			browseId: None,
			videoId: None,
			params: None,
			// These two could be `Some("foo")`
			inlineSettingStatus: None,
			// You might want to set this if you want to use transcriptions
			languageCode: None,
		}
	}
}

// The next two are for recommendations

impl Default for PlaybackContext {
	fn default() -> Self {
		// If you want to use this, you need to set `playbackContext:
		// Some(PlaybackContext::default())` in `NextRequest`
		Self {
			// This could be `Some(0)`
			vis: None,
			// This could be `Some("foo")`
			lactMilliseconds: None,
		}
	}
}

impl Default for NextRequest {
	fn default() -> Self {
		Self {
			context: Context::default(),
			// This is set by the program
			videoId: String::new(),
			// These two could be `Some(true)`
			racyCheckOk: None,
			contentCheckOk: None,
			// This could be `Some("foo")`
			autonavState: None,
			// This could be `Some(PlaybackContext::default())`
			playbackContext: None,
			// This could be `Some(true)`
			captionsRequested: None,
		}
	}
}

impl Default for SearchRequest {
	fn default() -> Self {
		Self {
			context: Context::default(),
			// These two are set by the program
			query: String::new(),
			params: None,
			// This coulf be `Some("foo")`
			webSearchboxStatsUrl: None,
		}
	}
}
