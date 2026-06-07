#[cfg(not(feature = "identify"))]
use chordclaw_core::{
    DEFAULT_MAX_SPAN, DEFAULT_MIN_FRET, VoicingMode, VoicingOptions, voicings_with_tuning,
};
#[cfg(feature = "identify")]
use chordclaw_core::{Fingering, IdentifyResult, identify_fingering_with_tuning};
use chordclaw_core::{GuitarTuning, Instrument};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "identify"))]
const MAX_CHORD_LEN: usize = 32;
#[cfg(feature = "identify")]
const MAX_FINGERING_LEN: usize = 96;
const MAX_TUNING_LEN: usize = 48;
#[cfg(not(feature = "identify"))]
const DEFAULT_MAX_FRET: u8 = 24;
#[cfg(not(feature = "identify"))]
const MAX_DEMO_FRET: u8 = 24;
#[cfg(not(feature = "identify"))]
const MAX_DEMO_SPAN: u8 = 5;
#[cfg(not(feature = "identify"))]
const MAX_DEMO_LIMIT: usize = 12;
#[cfg(not(feature = "identify"))]
const DEFAULT_DEMO_LIMIT: usize = 8;

#[cfg(not(feature = "identify"))]
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

#[cfg(not(feature = "identify"))]
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

#[cfg(not(feature = "identify"))]
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

#[cfg(not(feature = "identify"))]
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

#[cfg(not(feature = "identify"))]
fn generate_voicings(request_json: &str) -> Result<VoicingsResponse, String> {
    let request: VoicingsRequest =
        serde_json::from_str(request_json).map_err(|_| "Invalid ChordClaw request.".to_owned())?;

    let chord = request.chord.trim();
    if chord.is_empty() {
        return Err("Enter a chord.".to_owned());
    }
    if chord.len() > MAX_CHORD_LEN {
        return Err(format!(
            "Chord is too long. Maximum is {MAX_CHORD_LEN} characters."
        ));
    }

    let instrument_text = request.instrument.as_deref().unwrap_or("guitar").trim();
    let instrument = Instrument::parse(instrument_text).map_err(|error| error.to_string())?;
    let tuning = match request.tuning.as_deref().map(str::trim) {
        Some("") | None => instrument.default_tuning(),
        Some(tuning) => {
            if tuning.len() > MAX_TUNING_LEN {
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
    if fingering_text.len() > MAX_FINGERING_LEN {
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
            if tuning_text.len() > MAX_TUNING_LEN {
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
    if tuning_text.len() > MAX_TUNING_LEN {
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
