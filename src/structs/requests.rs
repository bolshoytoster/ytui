//! Structures sent in POST requests

#![allow(non_snake_case)]

use std::str::from_utf8_unchecked;

use curl::easy::Easy;
use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigInfo {
	pub appInstallData: Option<&'static str>,
}

#[derive(Serialize)]
pub struct MainAppWebInfo {
	pub graftUrl: Option<&'static str>,
	pub webDisplayMode: Option<&'static str>,
	pub isWebNativeShareAvailable: Option<bool>,
}

#[derive(Serialize)]
pub struct Client {
	pub hl: Option<&'static str>,
	pub gl: Option<&'static str>,
	pub remoteHost: Option<&'static str>,
	pub deviceMake: Option<&'static str>,
	pub deviceModel: Option<&'static str>,
	/// Required to get more than just the headers, set by the program
	pub visitorData: Option<String>,
	pub userAgent: Option<&'static str>,
	/// Should be "WEB"
	pub clientName: &'static str,
	/// Could be "2.0000011"
	pub clientVersion: &'static str,
	pub osName: Option<&'static str>,
	pub osVersion: Option<&'static str>,
	pub originalUrl: Option<&'static str>,
	pub platform: Option<&'static str>,
	pub clientFormFactor: Option<&'static str>,
	pub configInfo: Option<ConfigInfo>,
	pub browserName: Option<&'static str>,
	pub browserVersion: Option<&'static str>,
	pub acceptHeader: Option<&'static str>,
	pub deviceExperimentId: Option<&'static str>,
	pub screenWidthPoints: Option<u64>,
	pub screenHeightPoints: Option<u64>,
	pub screenPixelDensity: Option<u16>,
	pub screenDensityFloat: Option<u16>,
	pub utcOffsetMinutes: Option<i16>,
	pub userInterfaceTheme: Option<&'static str>,
	pub mainAppWebInfo: Option<MainAppWebInfo>,
	pub timeZone: Option<&'static str>,
}

#[derive(Serialize)]
pub struct User {
	pub lockedSafetyMode: Option<bool>,
}

#[derive(Serialize)]
pub struct Request {
	pub useSsl: Option<bool>,
	// I don't know what's in these arrays
	pub internalExperimentFlags: Option<Vec<()>>,
	pub consistencyTokenJars: Option<Vec<()>>,
}

#[derive(Serialize)]
pub struct Param {
	pub key: Option<&'static str>,
	pub value: Option<&'static str>,
}

#[derive(Serialize)]
pub struct AdSignalsInfo {
	pub params: Option<Vec<Param>>,
}

#[derive(Serialize)]
pub struct Context {
	pub client: Client,
	pub user: Option<User>,
	pub request: Option<Request>,
	pub adSignalsInfo: Option<AdSignalsInfo>,
}

/// A general request, used for many things
#[derive(Serialize)]
pub struct BrowseRequest {
	pub context: Context,
	pub continuation: Option<String>,
	pub browseId: Option<String>,
	pub inlineSettingStatus: Option<&'static str>,
	pub params: Option<String>,
	pub videoId: Option<String>,
	pub languageCode: Option<&'static str>,
}
impl BrowseRequest {
	/// Creates a new request.
	/// Takes the `__Secure-YEC` cookie from the given `Easy` session, returning `None` if it
	/// doesn't exist.
	pub fn new(easy: &mut Easy) -> Option<Self> {
		Some(Self {
			context: Context {
				client: Client {
					visitorData: Some(unsafe {
						String::from_utf8_unchecked(
							easy.cookies()
								.ok()?
								.iter()
								.find(|cookie| {
									from_utf8_unchecked(cookie).contains("__Secure-YEC")
								})?
								.rsplit(|x| *x == b'\t')
								.next()?
								.to_vec(),
						)
					}),
					..Client::default()
				},
				..Context::default()
			},
			..Self::default()
		})
	}
}

#[derive(Serialize)]
pub struct PlaybackContext {
	pub vis: Option<u8>,
	pub lactMilliseconds: Option<&'static str>,
}

/// A request used to get recommendations from a video
#[derive(Serialize)]
pub struct NextRequest {
	pub context: Context,
	pub videoId: String,
	pub racyCheckOk: Option<bool>,
	pub contentCheckOk: Option<bool>,
	pub autonavState: Option<&'static str>,
	pub playbackContext: Option<PlaybackContext>,
	pub captionsRequested: Option<bool>,
}

#[derive(Serialize)]
pub struct SearchRequest {
	pub context: Context,
	pub query: String,
	pub params: Option<String>,
	pub webSearchboxStatsUrl: Option<&'static str>,
}
