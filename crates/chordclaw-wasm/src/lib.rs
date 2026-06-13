#[cfg(feature = "voicings")]
use chordclaw_core::{
    DEFAULT_MAX_SPAN, DEFAULT_MIN_FRET, VoicingMode, VoicingOptions, voicings_with_tuning,
};
#[cfg(feature = "identify")]
use chordclaw_core::{Fingering, IdentifyResult, identify_fingering_with_tuning};
use chordclaw_core::{GuitarTuning, Instrument};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Install a panic hook on module load so a Rust panic surfaces a readable
/// message in the browser console instead of an opaque `unreachable` wasm trap.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(feature = "voicings")]
const MAX_CHORD_LEN: usize = 32;
#[cfg(feature = "identify")]
const MAX_FINGERING_LEN: usize = 96;
const MAX_TUNING_LEN: usize = 48;
#[cfg(feature = "voicings")]
const DEFAULT_MAX_FRET: u8 = 24;
#[cfg(feature = "voicings")]
const MAX_DEMO_FRET: u8 = 24;
#[cfg(feature = "voicings")]
const MAX_DEMO_SPAN: u8 = 5;
#[cfg(feature = "voicings")]
const MAX_DEMO_LIMIT: usize = 12;
#[cfg(feature = "voicings")]
const DEFAULT_DEMO_LIMIT: usize = 8;

#[cfg(feature = "voicings")]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VoicingsRequest {
    chord: String,
    instrument: Option<String>,
    tuning: Option<String>,
    min_fret: Option<u8>,
    max_fret: Option<u8>,
    max_span: Option<u8>,
    limit: Option<usize>,
}

#[cfg(feature = "identify")]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyRequest {
    fingering: String,
    instrument: Option<String>,
    tuning: Option<String>,
}

#[cfg(feature = "voicings")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VoicingsResponse {
    ok: bool,
    engine: &'static str,
    version: &'static str,
    query: VoicingsQuery,
    voicings: Vec<chordclaw_core::Voicing>,
}

#[cfg(feature = "identify")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyResponse {
    ok: bool,
    engine: &'static str,
    version: &'static str,
    query: IdentifyQuery,
    result: IdentifyResult,
}

#[cfg(feature = "voicings")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VoicingsQuery {
    chord: String,
    instrument: String,
    min_fret: u8,
    max_fret: u8,
    max_span: u8,
    limit: usize,
}

#[cfg(feature = "identify")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyQuery {
    fingering: String,
    instrument: String,
    tuning: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[wasm_bindgen]
pub fn chordclaw_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[cfg(feature = "voicings")]
#[wasm_bindgen]
pub fn chordclaw_voicings(request_json: &str) -> String {
    match generate_voicings(request_json) {
        Ok(response) => to_json(&response),
        Err(error) => error_json(error),
    }
}

#[cfg(feature = "identify")]
#[wasm_bindgen]
pub fn chordclaw_identify(request_json: &str) -> String {
    match identify_chord(request_json) {
        Ok(response) => to_json(&response),
        Err(error) => error_json(error),
    }
}

#[cfg(feature = "voicings")]
fn generate_voicings(request_json: &str) -> Result<VoicingsResponse, String> {
    let request: VoicingsRequest =
        serde_json::from_str(request_json).map_err(|_| "Invalid ChordClaw request.".to_owned())?;

    let chord = request.chord.trim();
    if chord.is_empty() {
        return Err("Enter a chord.".to_owned());
    }
    if chord.chars().count() > MAX_CHORD_LEN {
        return Err(format!(
            "Chord is too long. Maximum is {MAX_CHORD_LEN} characters."
        ));
    }

    let instrument_text = request.instrument.as_deref().unwrap_or("guitar").trim();
    let instrument = Instrument::parse(instrument_text).map_err(|error| error.to_string())?;
    let tuning = match request.tuning.as_deref().map(str::trim) {
        Some("") | None => instrument.default_tuning(),
        Some(tuning) => {
            if tuning.chars().count() > MAX_TUNING_LEN {
                return Err(format!(
                    "Tuning is too long. Maximum is {MAX_TUNING_LEN} characters."
                ));
            }
            GuitarTuning::parse_for_instrument(tuning, instrument)
                .map_err(|error| error.to_string())?
        }
    };

    let min_fret = request.min_fret.unwrap_or(DEFAULT_MIN_FRET);
    let max_fret = request.max_fret.unwrap_or(DEFAULT_MAX_FRET);
    let max_span = request.max_span.unwrap_or(DEFAULT_MAX_SPAN);
    let limit = request.limit.unwrap_or(DEFAULT_DEMO_LIMIT);

    if max_fret > MAX_DEMO_FRET {
        return Err(format!("Maximum fret for the demo is {MAX_DEMO_FRET}."));
    }
    if max_span > MAX_DEMO_SPAN {
        return Err(format!(
            "Maximum fret span for the demo is {MAX_DEMO_SPAN}."
        ));
    }
    if limit > MAX_DEMO_LIMIT {
        return Err(format!(
            "Maximum voicing limit for the demo is {MAX_DEMO_LIMIT}."
        ));
    }

    let options = VoicingOptions {
        min_fret,
        max_fret,
        max_span,
        mode: VoicingMode::Curated { limit },
    };
    let voicings =
        voicings_with_tuning(chord, tuning, options).map_err(|error| error.to_string())?;

    Ok(VoicingsResponse {
        ok: true,
        engine: "ChordClaw",
        version: env!("CARGO_PKG_VERSION"),
        query: VoicingsQuery {
            chord: chord.to_owned(),
            instrument: instrument_text.to_owned(),
            min_fret,
            max_fret,
            max_span,
            limit,
        },
        voicings,
    })
}

#[cfg(feature = "identify")]
fn identify_chord(request_json: &str) -> Result<IdentifyResponse, String> {
    let request: IdentifyRequest =
        serde_json::from_str(request_json).map_err(|_| "Invalid ChordClaw request.".to_owned())?;

    let fingering_text = request.fingering.trim();
    if fingering_text.is_empty() {
        return Err("Set at least one string.".to_owned());
    }
    if fingering_text.chars().count() > MAX_FINGERING_LEN {
        return Err(format!(
            "Fingering is too long. Maximum is {MAX_FINGERING_LEN} characters."
        ));
    }

    let (instrument, tuning) = identify_tuning(&request, fingering_text)?;
    let fingering = Fingering::parse_with_string_count(fingering_text, tuning.string_count())
        .map_err(|error| error.to_string())?;
    let result =
        identify_fingering_with_tuning(&fingering, tuning).map_err(|error| error.to_string())?;

    Ok(IdentifyResponse {
        ok: true,
        engine: "ChordClaw",
        version: env!("CARGO_PKG_VERSION"),
        query: IdentifyQuery {
            fingering: fingering.compact(),
            instrument: instrument.to_string(),
            tuning: tuning.to_string(),
        },
        result,
    })
}

#[cfg(feature = "identify")]
fn identify_tuning(
    request: &IdentifyRequest,
    fingering_text: &str,
) -> Result<(Instrument, GuitarTuning), String> {
    let instrument_text = request
        .instrument
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let tuning_text = request
        .tuning
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (instrument_text, tuning_text) {
        (Some(instrument_text), Some(tuning_text)) => {
            let instrument =
                Instrument::parse(instrument_text).map_err(|error| error.to_string())?;
            let tuning = parse_tuning_for_instrument(tuning_text, instrument)?;
            Ok((instrument, tuning))
        }
        (Some(instrument_text), None) => {
            let instrument =
                Instrument::parse(instrument_text).map_err(|error| error.to_string())?;
            Ok((instrument, instrument.default_tuning()))
        }
        (None, Some(tuning_text)) => {
            if tuning_text.chars().count() > MAX_TUNING_LEN {
                return Err(format!(
                    "Tuning is too long. Maximum is {MAX_TUNING_LEN} characters."
                ));
            }
            let tuning = GuitarTuning::parse(tuning_text).map_err(|error| error.to_string())?;
            Ok((tuning.instrument(), tuning))
        }
        (None, None) => {
            let string_count = Fingering::string_count_from_input(fingering_text)
                .map_err(|error| error.to_string())?;
            let instrument =
                Instrument::from_string_count(string_count).map_err(|error| error.to_string())?;
            Ok((instrument, instrument.default_tuning()))
        }
    }
}

#[cfg(feature = "identify")]
fn parse_tuning_for_instrument(
    tuning_text: &str,
    instrument: Instrument,
) -> Result<GuitarTuning, String> {
    if tuning_text.chars().count() > MAX_TUNING_LEN {
        return Err(format!(
            "Tuning is too long. Maximum is {MAX_TUNING_LEN} characters."
        ));
    }
    GuitarTuning::parse_for_instrument(tuning_text, instrument).map_err(|error| error.to_string())
}

fn error_json(error: String) -> String {
    to_json(&ErrorResponse { ok: false, error })
}

fn to_json<T: Serialize>(value: &T) -> String {
    match serde_json::to_string(value) {
        Ok(json) => json,
        Err(_) => "{\"ok\":false,\"error\":\"Failed to encode ChordClaw response.\"}".to_owned(),
    }
}

// The host-side JSON contract is what the website consumes; cover it directly.
// The cdylib+rlib crate-type lets these run under a normal `cargo test`.
// Each suite is gated on the feature that exposes the API it exercises.
#[cfg(all(test, feature = "voicings"))]
mod voicings_tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn rejects_invalid_request_json() {
        let json = chordclaw_voicings("not json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], false);
        assert!(value["error"].is_string());
    }

    #[test]
    fn rejects_empty_chord() {
        let json = chordclaw_voicings(r#"{"chord":"   "}"#);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], false);
    }

    #[test]
    fn rejects_limit_above_demo_cap() {
        let json = chordclaw_voicings(r#"{"chord":"C","limit":9999}"#);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], false);
        assert!(
            value["error"]
                .as_str()
                .unwrap_or_default()
                .contains("voicing limit")
        );
    }

    #[test]
    fn ok_response_carries_engine_and_voicings() {
        let json = chordclaw_voicings(r#"{"chord":"C"}"#);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], true);
        assert_eq!(value["engine"], "ChordClaw");
        assert!(value["voicings"].is_array());
        assert_eq!(value["query"]["chord"], "C");
    }
}

#[cfg(all(test, feature = "identify"))]
mod identify_tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn rejects_invalid_request_json() {
        let json = chordclaw_identify("not json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], false);
        assert!(value["error"].is_string());
    }

    #[test]
    fn rejects_empty_fingering() {
        let json = chordclaw_identify(r#"{"fingering":"   "}"#);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], false);
    }

    #[test]
    fn ok_response_names_open_c_major() {
        let json = chordclaw_identify(r#"{"fingering":"x32010","instrument":"guitar"}"#);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["ok"], true);
        assert_eq!(value["engine"], "ChordClaw");
        assert_eq!(value["result"]["primary"]["symbol"], "C");
    }
}
