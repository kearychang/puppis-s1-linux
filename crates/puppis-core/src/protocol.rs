use crate::{DeviceIdentity, DeviceRole, OperationFailure, RadioBand, RadioConfiguration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const MAGIC: u8 = 0xaa;
const REQUEST_TYPE: u8 = 0x01;
const RESPONSE_TYPE: u8 = 0x02;
const RESERVED: u8 = 0x00;
static CONNECTION: OnceLock<Mutex<Option<TcpStream>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolResponse {
    #[serde(rename = "fun")]
    pub function: String,
    #[serde(default)]
    pub data: Value,
    pub status: String,
}

#[derive(Serialize)]
struct Request<'a> {
    fun: &'a str,
    args: Value,
}

fn encode_getter(function: &str) -> Result<Vec<u8>, OperationFailure> {
    encode_request(function, Value::Object(Default::default()))
}

fn encode_command(function: &str, arguments: Value) -> Result<Vec<u8>, OperationFailure> {
    encode_request(function, arguments)
}

fn encode_request(function: &str, arguments: Value) -> Result<Vec<u8>, OperationFailure> {
    let json = serde_json::to_vec(&Request {
        fun: function,
        args: arguments,
    })
    .map_err(|_| invalid("The device request could not be encoded."))?;
    let total_length = json.len() + 8;
    if total_length > u8::MAX as usize {
        return Err(invalid(
            "The device request exceeds the qualified frame size.",
        ));
    }
    let mut body = Vec::with_capacity(json.len() + 2);
    body.push(REQUEST_TYPE);
    body.push(RESERVED);
    body.extend_from_slice(&json);
    let mut frame = Vec::with_capacity(total_length);
    frame.push(MAGIC);
    frame.push(total_length as u8);
    frame.extend_from_slice(&crc32(&body).to_be_bytes());
    frame.extend_from_slice(&body);
    Ok(frame)
}

fn decode_response(
    frame: &[u8],
    expected_function: &str,
) -> Result<ProtocolResponse, OperationFailure> {
    if frame.len() < 8
        || frame[0] != MAGIC
        || frame[1] as usize != frame.len()
        || frame[6] != RESPONSE_TYPE
        || frame[7] != RESERVED
        || u32::from_be_bytes(frame[2..6].try_into().expect("four CRC bytes")) != crc32(&frame[6..])
    {
        return Err(invalid("The Puppis returned an invalid protocol frame."));
    }
    let json = std::str::from_utf8(&frame[8..])
        .map_err(|_| invalid("The Puppis returned invalid text."))?;
    let response: ProtocolResponse = serde_json::from_str(json)
        .map_err(|_| invalid("The Puppis returned an invalid response object."))?;
    if response.function != expected_function {
        return Err(OperationFailure::safe(
            "protocol_response_mismatch",
            "The Puppis response did not match the requested operation.",
            "Reconnect the device and try the read-only operation again.",
        ));
    }
    Ok(response)
}

fn invalid(message: &str) -> OperationFailure {
    OperationFailure::safe(
        "protocol_invalid_frame",
        message,
        "This firmware may be incompatible. Device mutations remain unavailable.",
    )
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

pub fn read_device_identity() -> Result<DeviceIdentity, OperationFailure> {
    let response = exchange_getter("getDevice")?;
    ensure_success(&response)?;
    Ok(DeviceIdentity {
        model: required_string(&response.data, "model")?,
        firmware: required_string(&response.data, "fw")?,
        role_code: required_string(&response.data, "mode")?,
    })
}

pub fn read_radio(band: RadioBand) -> Result<RadioConfiguration, OperationFailure> {
    let function = match band {
        RadioBand::FiveGhz => "get5GHotspot",
        RadioBand::TwoPointFourGhz => "get2GHotspot",
    };
    let response = exchange_getter(function)?;
    parse_radio_response(&response)
}

fn parse_radio_response(
    response: &ProtocolResponse,
) -> Result<RadioConfiguration, OperationFailure> {
    ensure_success(response)?;
    let radio = RadioConfiguration {
        ssid: required_string(&response.data, "ssid")?,
        password: required_string_allow_empty(&response.data, "pwd")?,
        protection: required_string(&response.data, "pt")?,
        channel: required_string(&response.data, "ch")?,
        country: required_string(&response.data, "code")?,
        enabled: required_string(&response.data, "en")?,
        encryption: required_string(&response.data, "encrypt")?,
        bandwidth: required_string(&response.data, "bw")?,
    };
    let numeric = |value: &str| value.bytes().all(|byte| byte.is_ascii_digit());
    let valid = radio.ssid.len() <= 32
        && numeric(&radio.protection)
        && numeric(&radio.channel)
        && radio.country.len() == 2
        && radio.country.bytes().all(|byte| byte.is_ascii_uppercase())
        && matches!(radio.enabled.as_str(), "0" | "1")
        && numeric(&radio.encryption)
        && numeric(&radio.bandwidth);
    if !valid {
        return Err(invalid(
            "The Puppis returned radio settings outside the validated schema.",
        ));
    }
    Ok(radio)
}

fn ensure_success(response: &ProtocolResponse) -> Result<(), OperationFailure> {
    if response.status == "ok" {
        Ok(())
    } else {
        Err(invalid(
            "The Puppis returned an unsuccessful getter response.",
        ))
    }
}

fn required_string(data: &Value, key: &str) -> Result<String, OperationFailure> {
    let value = required_string_allow_empty(data, key)?;
    if value.is_empty() {
        Err(invalid("The Puppis response omitted a required setting."))
    } else {
        Ok(value)
    }
}

fn required_string_allow_empty(data: &Value, key: &str) -> Result<String, OperationFailure> {
    data.as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid("The Puppis response omitted a required setting."))
}

pub fn write_radio(band: RadioBand, radio: &RadioConfiguration) -> Result<(), OperationFailure> {
    let function = match band {
        RadioBand::FiveGhz => "set5GHotspot",
        RadioBand::TwoPointFourGhz => "set2GHotspot",
    };
    let arguments = serde_json::json!({
        "ssid": radio.ssid, "pwd": radio.password, "pt": radio.protection, "ch": radio.channel,
        "code": radio.country, "en": radio.enabled, "encrypt": radio.encryption, "bw": radio.bandwidth,
    });
    let response = exchange_frame(&encode_command(function, arguments)?, function, false)?;
    if response.status == "ok" {
        Ok(())
    } else {
        Err(OperationFailure::safe(
            "device_mutation_rejected",
            "The Puppis rejected the requested settings transaction.",
            "The manager will reconcile the observed state before another mutation.",
        ))
    }
}

pub fn write_role(role: DeviceRole) -> Result<(), OperationFailure> {
    let code = match role {
        DeviceRole::PrismPulse => "1",
        DeviceRole::WifiHotspot => "2",
        DeviceRole::WifiAdapter => "3",
    };
    let response = exchange_frame(
        &encode_command("setMode", serde_json::json!({ "mode": code }))?,
        "setMode",
        false,
    )?;
    if response.status == "ok" {
        Ok(())
    } else {
        Err(OperationFailure::safe(
            "device_mutation_rejected",
            "The Puppis rejected the requested role transition.",
            "The manager will reconcile the observed role.",
        ))
    }
}

fn exchange_getter(function: &str) -> Result<ProtocolResponse, OperationFailure> {
    exchange_frame(&encode_getter(function)?, function, true)
}

pub(crate) fn disconnect() {
    if let Some(connection) = CONNECTION.get() {
        *connection.lock().expect("P1411 connection lock poisoned") = None;
    }
}

fn exchange_frame(
    frame: &[u8],
    function: &str,
    retry_getter_once: bool,
) -> Result<ProtocolResponse, OperationFailure> {
    let attempts = if retry_getter_once { 2 } else { 1 };
    let mut last = None;
    for _ in 0..attempts {
        match exchange_once(frame, function) {
            Ok(response) => return Ok(response),
            Err(error) => last = Some(error),
        }
    }
    Err(last.unwrap_or_else(unreachable_failure))
}

fn exchange_once(frame: &[u8], function: &str) -> Result<ProtocolResponse, OperationFailure> {
    let connection = CONNECTION.get_or_init(|| Mutex::new(None));
    let mut connection = connection.lock().expect("P1411 connection lock poisoned");
    if connection.is_none() {
        *connection = Some(connect()?);
    }
    let result = exchange_on_stream(
        connection.as_mut().expect("connection initialized"),
        frame,
        function,
    );
    if result.is_err() {
        *connection = None;
    }
    result
}

fn connect() -> Result<TcpStream, OperationFailure> {
    let endpoint = SocketAddrV4::new(Ipv4Addr::new(192, 168, 137, 254), 10081);
    let stream = TcpStream::connect_timeout(&endpoint.into(), Duration::from_secs(3))
        .map_err(|_| unreachable_failure())?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();
    Ok(stream)
}

fn exchange_on_stream(
    stream: &mut TcpStream,
    frame: &[u8],
    function: &str,
) -> Result<ProtocolResponse, OperationFailure> {
    stream.write_all(frame).map_err(|_| unreachable_failure())?;
    let mut header = [0u8; 8];
    stream
        .read_exact(&mut header)
        .map_err(|_| unreachable_failure())?;
    let total = usize::from(header[1]);
    if total < 8 {
        return Err(invalid("The Puppis returned an invalid frame length."));
    }
    let mut frame = Vec::with_capacity(total);
    frame.extend_from_slice(&header);
    frame.resize(total, 0);
    stream
        .read_exact(&mut frame[8..])
        .map_err(|_| unreachable_failure())?;
    decode_response(&frame, function)
}

fn unreachable_failure() -> OperationFailure {
    OperationFailure::safe(
        "protocol_unreachable",
        "The Puppis protocol is not reachable.",
        "Check the Puppis-facing host network and try again.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QUALIFIED_FIRMWARE;

    const GET_DEVICE_REQUEST: &str =
        "aa25bd52005d01007b2266756e223a22676574446576696365222c2261726773223a7b7d7d";
    const REDACTED_DEVICE_RESPONSE: &str = "aac887937b9402007b2266756e223a22676574446576696365222c2264617461223a7b226d6f6465223a2231222c22616c696173223a225031343131222c226d6f64656c223a225031343131222c22706e223a22507269736d585220507570706973205331222c22736e223a225b52454441435445445d222c226677223a22422d4d443246503134313156312e32322d3235303130382d7230653137222c2273747265616d223a2231222c227374696d65223a22373636227d2c22737461747573223a226f6b227d";

    #[test]
    fn captured_getter_and_response_pass_the_strict_protocol_contract() {
        assert_eq!(
            hex(&encode_getter("getDevice").unwrap()),
            GET_DEVICE_REQUEST
        );
        let response = decode_response(&unhex(REDACTED_DEVICE_RESPONSE), "getDevice").unwrap();
        assert_eq!(response.data["model"], "P1411");
        assert_eq!(response.data["fw"], QUALIFIED_FIRMWARE);
    }

    #[test]
    fn every_untrusted_frame_boundary_is_rejected() {
        let valid = unhex(REDACTED_DEVICE_RESPONSE);
        let cases = vec![
            changed(&valid, 0, 0xab),
            changed(&valid, 1, 8),
            changed(&valid, 2, valid[2] ^ 1),
            changed(&valid, 6, 1),
            changed(&valid, 7, 1),
            reframed(&[0xff, 0xfe]),
        ];
        for frame in cases {
            assert_eq!(
                decode_response(&frame, "getDevice").unwrap_err().code,
                "protocol_invalid_frame"
            );
        }
        assert_eq!(
            decode_response(&valid, "get5GHotspot").unwrap_err().code,
            "protocol_response_mismatch"
        );
    }

    #[test]
    fn incomplete_or_wrongly_typed_radio_objects_are_rejected_before_preservation() {
        let complete = serde_json::json!({
            "ssid": "five", "pwd": "secret", "pt": "0", "ch": "36", "code": "CA",
            "en": "0", "encrypt": "5", "bw": "160"
        });
        let response = |data| ProtocolResponse {
            function: "get5GHotspot".into(),
            data,
            status: "ok".into(),
        };
        assert!(parse_radio_response(&response(complete.clone())).is_ok());
        let mut missing = complete.clone();
        missing.as_object_mut().unwrap().remove("encrypt");
        assert!(parse_radio_response(&response(missing)).is_err());
        let mut wrong_type = complete;
        wrong_type["bw"] = serde_json::json!(160);
        assert!(parse_radio_response(&response(wrong_type)).is_err());
    }

    fn changed(frame: &[u8], index: usize, byte: u8) -> Vec<u8> {
        let mut changed = frame.to_vec();
        changed[index] = byte;
        changed
    }

    fn reframed(json: &[u8]) -> Vec<u8> {
        let mut body = vec![2, 0];
        body.extend_from_slice(json);
        let mut frame = vec![0xaa, (body.len() + 6) as u8];
        frame.extend_from_slice(&crc32(&body).to_be_bytes());
        frame.extend_from_slice(&body);
        frame
    }

    fn unhex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    fn hex(value: &[u8]) -> String {
        value.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
