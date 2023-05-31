//! Data returned from requesting a video/stream

#![allow(non_snake_case)]

use std::cmp::Ordering;
use std::io::stdout;
use std::process::Command;

use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use js_sandbox::Script;
use serde::Deserialize;
use urldecode::decode;

use super::SimpleText;
use crate::config::*;

/// Formats a number of seconds to be human readable
fn seconds_to_human(seconds: String) -> String {
	// This is needed since expressions can't be used in match conditions
	const MINUTE: i64 = 60;
	const HOUR: i64 = 60 * MINUTE;
	const DAY: i64 = 24 * HOUR;
	const MONTH: i64 = 365 / 12 * DAY;
	const YEAR: i64 = 365 * DAY;

	let parsed = seconds.parse().expect("Length should be a number");

	match parsed {
		..MINUTE => [&parsed.to_string(), " Seconds"].concat(),
		MINUTE..HOUR => [&(parsed / MINUTE).to_string(), " Minutes"].concat(),
		HOUR..DAY => [&(parsed / HOUR).to_string(), " Hours"].concat(),
		DAY..MONTH => [&(parsed / DAY).to_string(), " Days"].concat(),
		MONTH..YEAR => [&(parsed / MONTH).to_string(), " Months"].concat(),
		YEAR.. => [&(parsed / YEAR).to_string(), " Years"].concat(),
	}
}

/// Deciphers the given signature, calling the `s` function in the given `Script`
fn decipher_signature(script: &mut Script, signature_cipher: String) -> String {
	// Iter over params in the string, assuming they're in alphabetical
	// order (`s`, `sp`, `url`).
	let mut split = signature_cipher.split('&');

	let url = &split.next_back().unwrap()[4..];
	let sp = &split.next_back().unwrap()[3..];
	let s = decode(split.next_back().unwrap()[2..].to_owned());

	[
		// Url decode two levels
		&decode(url.replace("%25", "%")),
		"&",
		sp,
		"=",
		&script
			.call::<_, String>(
				"s",
				&('\0'..unsafe { char::from_u32_unchecked(s.len() as u32) }).collect::<String>(),
			)
			.expect("`sig` function shouldn't fail")
			.bytes()
			.map(|i| {
				s.chars()
					.nth(i.into())
					.expect("Indices from the `sig` function should be within range")
			})
			.collect::<String>(),
	]
	.concat()
}

#[derive(Deserialize)]
struct CaptionTrack {
	baseUrl: String,
	name: SimpleText,
	// Ignore `vssId`, `languageCode`, `isTranslatable`
}

#[derive(Deserialize)]
struct PlayerCaptionsTracklistRenderer {
	captionTracks: Vec<CaptionTrack>,
	// Ignore `audioTracks`, `defaultAudioTrackIndex`, `openTranscriptCommand`,
	// `translationLanguages`
}

#[derive(Deserialize)]
struct Captions {
	playerCaptionsTracklistRenderer: PlayerCaptionsTracklistRenderer,
}

#[derive(Deserialize)]
struct PlayerMicroformatRenderer {
	title: SimpleText,
	description: Option<SimpleText>,
	lengthSeconds: String,
	/// It means "family friendly", hopefully their family is always safe.
	isFamilySafe: bool,
	isUnlisted: bool,
	viewCount: String,
	category: String,
	ownerChannelName: String,
	uploadDate: String,
	// Ignore `thumbnail`, `embed`, `ownerProfileUrl`, `externalChannelId`, `availableCountries`,
	// `HasYpcMetadata`, `publishDate`
}

#[derive(Deserialize)]
struct Microformat {
	playerMicroformatRenderer: PlayerMicroformatRenderer,
}

#[derive(Deserialize)]
struct AudioTrack {
	displayName: String,
	// Ignore `audioIsDefault`, `id`
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AdaptiveFormat {
	Video {
		bitrate: u32,
		height: u16,
		mimeType: String,
		url: String,
		// Ignore `approxDurationMs`, `averageBitrate`, `contentLength`, `fps`, `indexRange`,
		// `initRange`, `itag`, `lastModified`, `projectionType`, `quality`, `qualityLabel`,
		// `width`
	},
	/// Some videos will have a ciphered url
	VideoCipher {
		bitrate: u32,
		height: u16,
		mimeType: String,
		signatureCipher: String,
	},
	Audio {
		audioTrack: Option<AudioTrack>,
		bitrate: u32,
		mimeType: String,
		url: String,
		// Ignore `approxDurationMs`, `audioChannels`, `audioQuality`, `audioSampleRate`,
		// `averageBitrate`, `contentLength`, `indexRange`, `initRange`, `itag`, `lastModified`,
		// `loudnessDb`, `projectionType`, `quality`, `xtags`
	},
	/// Some videos will have a ciphered url
	AudioCipher {
		audioTrack: Option<AudioTrack>,
		bitrate: u32,
		mimeType: String,
		signatureCipher: String,
	},
}
impl AdaptiveFormat {
	/// Returns the bitrate
	fn bitrate(&self) -> u32 {
		let (Self::Video { bitrate, .. }
		| Self::VideoCipher { bitrate, .. }
		| Self::Audio { bitrate, .. }
		| Self::AudioCipher { bitrate, .. }) = self;

		*bitrate
	}

	/// Returns the mime type
	fn mime_type(&self) -> &String {
		let (Self::Video { mimeType, .. }
		| Self::VideoCipher { mimeType, .. }
		| Self::Audio { mimeType, .. }
		| Self::AudioCipher { mimeType, .. }) = self;

		mimeType
	}

	/// Returns this video's height, panicing if this is audio
	fn height(&self) -> u16 {
		#[rustfmt::skip]
		let (Self::Video { height, .. }
        | Self::VideoCipher { height, .. }) = self else {
            panic!("Tried to get height of audio")
        };

		*height
	}

	/// Returns the audio track, if it exists and this isn't video
	fn audio_track(&self) -> &Option<AudioTrack> {
		if let Self::Audio { audioTrack, .. } | Self::AudioCipher { audioTrack, .. } = self {
			audioTrack
		} else {
			&None
		}
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StreamingData {
	Stream {
		hlsManifestUrl: String,
		// Ignore `adaptiveFormats`, `dashManifestUrl` and `expiresInSeconds`
	},
	Video {
		adaptiveFormats: Vec<AdaptiveFormat>,
		// Ignore `expiresInSeconds` and `formats`
	},
}

#[derive(Deserialize)]
struct VideoDetails {
	videoId: String,
	// Ignore `title`, `lengthSeconds`, `channelId`, `isOwnerViewing`, `shortDescription`,
	// `isCrawlable`, `thumbnail`, `allowRatings`, `viewCount`, `author`, `isPrivate`,
	// `isUnpluggedCorpus` and `isLiveContent`
}

/// A youtube video
#[derive(Deserialize)]
pub struct VideoResponse {
	captions: Option<Captions>,
	microformat: Microformat,
	streamingData: StreamingData,
	videoDetails: VideoDetails,
	// Ignore `adPlacements`, `attestation`, `cards`, `frameworkUpdates`, `playabilityStatus`,
	// `playbackTracking`, `playerAds`, `playerConfig`, `responseContext`, `storyboards`,
	// `trackingParams`
}
impl VideoResponse {
	/// Play this video using the user's config
	pub fn play(self, script: &mut Script) {
		let _ = disable_raw_mode();

		// We want to be in a normal terminal
		let _ = execute!(stdout(), LeaveAlternateScreen);

		// Print some video info
		println!(
			"Title: {}",
			self.microformat.playerMicroformatRenderer.title.simpleText
		);

		if let Some(description) = self.microformat.playerMicroformatRenderer.description {
			println!("Description: {}", description.simpleText);
		}

		println!(
			"
Length: {}
{}amily friendly
{}nlisted
Views: {}
Category: {}
Uploader: {}
Uploaded: {}
",
			seconds_to_human(self.microformat.playerMicroformatRenderer.lengthSeconds),
			if self.microformat.playerMicroformatRenderer.isFamilySafe {
				"F"
			} else {
				"Not f"
			},
			if self.microformat.playerMicroformatRenderer.isUnlisted {
				"U"
			} else {
				"Not u"
			},
			self.microformat.playerMicroformatRenderer.viewCount,
			self.microformat.playerMicroformatRenderer.category,
			self.microformat.playerMicroformatRenderer.ownerChannelName,
			self.microformat.playerMicroformatRenderer.uploadDate
		);

		let _ = match self.streamingData {
			StreamingData::Video { adaptiveFormats } => {
				// Video, we need to pick the appropriate URL

				// Iter the available video/audio tracks
				let mut ideal_video = None;
				let mut ideal_audio = None;
				for adaptive_format in adaptiveFormats {
					match adaptive_format {
						AdaptiveFormat::Video { .. } | AdaptiveFormat::VideoCipher { .. } => {
							let found = !ideal_video.as_ref().is_some_and(|video| {
								#[rustfmt::skip]
								let (AdaptiveFormat::Video {
                                    bitrate,
                                    height,
                                    mimeType,
                                    ..
                                } | AdaptiveFormat::VideoCipher {
                                    bitrate,
                                    height,
                                    mimeType,
                                    ..
                                }) = video else {
                                    unreachable!()
                                };

								// Go through the selectors in the user's config
								for video_selector in VIDEO_SELECTOR {
									match match video_selector {
										VideoSelector::Bitrate(selector) => {
											selector.is_better(*bitrate, adaptive_format.bitrate())
										}
										VideoSelector::Format(format) => adaptive_format
											.mime_type()
											.contains::<&str>(&format.to_string())
											.cmp(&mimeType.contains::<&str>(&format.to_string())),
										VideoSelector::Quality(selector) => {
											selector.is_better(*height, adaptive_format.height())
										}
									} {
										// The new video is worse than the
										// selected
										Ordering::Less => return true,
										// This video is equal, check the next
										// selector
										Ordering::Equal => (),
										// This video is better, let's select it
										Ordering::Greater => return false,
									}
								}

								// This video is equal to the selected one
								true
							});

							if found {
								ideal_video = Some(adaptive_format);
							}
						}
						AdaptiveFormat::Audio { .. } | AdaptiveFormat::AudioCipher { .. } => {
							let found = !ideal_audio.as_ref().is_some_and(|audio| {
								#[rustfmt::skip]
								let (AdaptiveFormat::Audio {
                                    audioTrack,
                                    bitrate,
                                    mimeType,
                                    ..
                                } | AdaptiveFormat::AudioCipher {
                                    audioTrack,
                                    bitrate,
                                    mimeType,
                                    ..
                                }) = audio else {
                                    unreachable!()
                                };

								// Go through the selectors in the user's config
								for audio_selector in AUDIO_SELECTOR {
									match match audio_selector {
										AudioSelector::Bitrate(selector) => {
											selector.is_better(*bitrate, adaptive_format.bitrate())
										}
										AudioSelector::Format(format) => adaptive_format
											.mime_type()
											.contains::<&str>(&format.to_string())
											.cmp(&mimeType.contains::<&str>(&format.to_string())),
										AudioSelector::Language(language) => adaptive_format
											.audio_track()
											.as_ref()
											.is_some_and(|x| &x.displayName == language)
											.cmp(
												&audioTrack
													.as_ref()
													.is_some_and(|x| &x.displayName == language),
											),
									} {
										// The new video is worse than the
										// selected
										Ordering::Less => return false,
										// This video is equal, check the next
										// selector
										Ordering::Equal => (),
										// This video is better, let's select it
										Ordering::Greater => return true,
									}
								}

								// This video is equal to the selected one
								false
							});

							if found {
								ideal_audio = Some(adaptive_format);
							}
						}
					}
				}

				// The same n is used for video and audio, we cache it
				#[allow(unused_assignments)]
				let mut decrypted_n = None;

				let (program, args) = video_player(
					match ideal_video.expect("Should be at least one video track") {
						AdaptiveFormat::Video { mut url, .. } => {
							// Extract n parameter from URL
							let n_start = url
								.find("&n=")
								.expect("Video URL should contain `n` challenge")
								+ 3;

							let n_end = n_start + url[n_start..].find('&').unwrap_or(url.len());

							// Eval `n` function and store result
							decrypted_n = Some(
								script
									.call::<_, String>("f", &&url[n_start..n_end])
									.expect("`n` function shouldn't fail"),
							);

							url.replace_range(
								n_start..n_end,
								decrypted_n.as_ref().expect("We just set it"),
							);

							url
						}
						AdaptiveFormat::VideoCipher {
							signatureCipher, ..
						} => decipher_signature(script, signatureCipher),
						_ => unreachable!("`ideal_video` should always be a video"),
					},
					match ideal_audio.expect("Should be at least one audio track") {
						AdaptiveFormat::Audio { mut url, .. } => {
							let start = url
								.find("&n=")
								.expect("Audio URL should contain `n` challenge")
								+ 3;

							url.replace_range(
								start..start + url[start..].find('&').unwrap_or(url.len()),
								&decrypted_n.expect("`n` challenge should be solved"),
							);

							url
						}
						AdaptiveFormat::AudioCipher {
							signatureCipher, ..
						} => decipher_signature(script, signatureCipher),
						_ => {
							unreachable!("`ideal_audio` should always be audio")
						}
					},
					self.captions
						.zip_with(CAPTION_LANGUAGE, |captions, language| {
							captions
								.playerCaptionsTracklistRenderer
								.captionTracks
								.into_iter()
								.find(|track| track.name.simpleText == language)
								.map(|track| track.baseUrl)
						})
						.flatten(),
				);

				Command::new(program).args(args).spawn()
			}
			StreamingData::Stream { hlsManifestUrl } => {
				// It's a stream, the player can handle the resolution stuff
				let (program, args) = stream_player(hlsManifestUrl);

				Command::new(program).args(args).spawn()
			}
		}
		.unwrap_or_else(|_| panic!("Should be able to spawn PLAYER"))
		.wait();

		let _ = enable_raw_mode();
		let _ = execute!(stdout(), EnterAlternateScreen);
	}
}
