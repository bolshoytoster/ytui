//! Useful functions that are used in multiple files in the program

use std::io::Read;

use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use simd_json::{from_slice, to_vec};

/// Send a GET request and return it as a `Vec<u8>`.
pub fn request_get(easy: &mut Easy, url: &str) -> Vec<u8> {
	let mut vec = Vec::new();

	let _ = easy.url(url);
	let _ = easy.get(true);

	// Make sure `transfer` is dropped before we use can `vec` again
	{
		let mut transfer = easy.transfer();

		let _ = transfer.write_function(|slice| {
			// Copy the packet to the buffer
			vec.extend_from_slice(slice);
			Ok(slice.len())
		});

		let _ = transfer.perform();
	}

	vec
}

/// Send a POST request and return it as a `Vec<u8>`.
pub fn request_post(easy: &mut Easy, url: &str, json: &(impl Serialize + ?Sized)) -> Vec<u8> {
	let mut data = &*to_vec(json).expect("Should be able to serialize POST data");

	let _ = easy.url(url);
	let _ = easy.post(true);

	let mut vec = Vec::new();

	// Make sure `transfer` is dropped before we use can `vec` again
	{
		let mut transfer = easy.transfer();

		let _ = transfer.read_function(|slice| Ok(data.read(slice).unwrap_or(0)));
		let _ = transfer.write_function(|slice| {
			// Copy the packet to the buffer
			vec.extend_from_slice(slice);
			Ok(slice.len())
		});

		let _ = transfer.perform();
	}

	vec
}

/// Extracts string between two substrings and parses it as JSON
/// The `overshoot` parameter is how far the end string overshoots the actual data, i.e. if it
/// includes a semicolon afterwards
pub fn extract_json<'a, J: Deserialize<'a>>(
	string: &'a mut str,
	start_str: &str,
	end_str: &str,
	overshoot: usize,
) -> Option<J> {
	let start = string.find(start_str)?;

	let end = start + string[start..].find(end_str)? + end_str.len() - overshoot;

	from_slice(unsafe { string[start..end].as_bytes_mut() }).ok()
}
