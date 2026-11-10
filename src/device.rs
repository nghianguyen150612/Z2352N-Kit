use serde::Serialize;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);
const OUTPUT_LIMIT: usize = 32 * 1024;
const PROPERTY_LIMIT: usize = 128;

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub success: bool,
    pub stdout: String,
}

pub trait Runner {
    fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, String>;
}

pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, String> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot run {program}: {e}"))?;

        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("{program} timed out after 8 seconds"));
                }
                Ok(None) => thread::sleep(Duration::from_millis(40)),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("{program} wait error: {e}"));
                }
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("{program} output error: {e}"))?;
        if output.stdout.len() > OUTPUT_LIMIT || output.stderr.len() > OUTPUT_LIMIT {
            return Err(format!("{program} output exceeded safety limit"));
        }

        Ok(ProcessOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct HostTools {
    pub adb: bool,
    pub fastboot: bool,
    pub lsusb: bool,
    pub python3: bool,
}

impl HostTools {
    pub fn detect() -> Self {
        Self {
            adb: in_path("adb"),
            fastboot: in_path("fastboot"),
            lsusb: in_path("lsusb"),
            python3: in_path("python3"),
        }
    }
}

fn in_path(binary: &str) -> bool {
    env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .any(|dir| Path::new(&dir).join(binary).is_file())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    Adb,
    Fastboot,
    Disconnected,
    Unauthorized,
    Offline,
    MultipleDevices,
}

impl Connection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Adb => "adb",
            Self::Fastboot => "fastboot",
            Self::Disconnected => "disconnected",
            Self::Unauthorized => "unauthorized",
            Self::Offline => "offline",
            Self::MultipleDevices => "multiple_devices",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Identity {
    ReportedZ2352n,
    NotConfirmed,
}

impl Identity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReportedZ2352n => "reported_z2352n",
            Self::NotConfirmed => "not_confirmed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LockAssessment {
    ReportedUnlocked,
    ReportedLocked,
    Inconsistent,
    InsufficientEvidence,
}

impl LockAssessment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReportedUnlocked => "reported_unlocked (not independently verified)",
            Self::ReportedLocked => "reported_locked (not independently verified)",
            Self::Inconsistent => "inconsistent_properties",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LockEvidence {
    pub flash_locked: Option<String>,
    pub vbmeta_device_state: Option<String>,
    pub verified_boot_state: Option<String>,
    pub assessment: LockAssessment,
}

impl LockEvidence {
    fn new(flash: Option<String>, vbmeta: Option<String>, verified: Option<String>) -> Self {
        let assessment = classify_lock(flash.as_deref(), vbmeta.as_deref(), verified.as_deref());
        Self {
            flash_locked: flash,
            vbmeta_device_state: vbmeta,
            verified_boot_state: verified,
            assessment,
        }
    }
}

fn classify_lock(
    flash: Option<&str>,
    vbmeta: Option<&str>,
    verified: Option<&str>,
) -> LockAssessment {
    match (flash, vbmeta, verified) {
        (Some("0"), Some("unlocked"), Some("orange")) => LockAssessment::ReportedUnlocked,
        (Some("1"), Some("locked"), Some("green" | "yellow")) => LockAssessment::ReportedLocked,
        (Some("0" | "1"), Some("unlocked" | "locked"), Some("green" | "yellow" | "orange")) => {
            LockAssessment::Inconsistent
        }
        _ => LockAssessment::InsufficientEvidence,
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceProbe {
    pub connection: Connection,
    pub identity: Identity,
    pub model: Option<String>,
    pub board: Option<String>,
    pub device_name: Option<String>,
    pub android_version: Option<String>,
    pub lock: LockEvidence,
    pub notes: Vec<String>,
}

impl DeviceProbe {
    fn empty(connection: Connection, notes: Vec<String>) -> Self {
        Self {
            connection,
            identity: Identity::NotConfirmed,
            model: None,
            board: None,
            device_name: None,
            android_version: None,
            lock: LockEvidence::new(None, None, None),
            notes,
        }
    }
}

/// Never records or returns USB serials, IMEI, or Android IDs.
pub fn probe(runner: &dyn Runner) -> DeviceProbe {
    let devices = runner
        .run("adb", &["devices"])
        .ok()
        .filter(|o| o.success)
        .map(|o| parse_adb_states(&o.stdout))
        .unwrap_or_default();

    if devices.len() > 1 {
        return DeviceProbe::empty(
            Connection::MultipleDevices,
            vec!["More than one ADB device found; disconnect extras before probing.".into()],
        );
    }

    if let Some(state) = devices.first() {
        return match state.as_str() {
            "device" => probe_adb(runner),
            "unauthorized" => DeviceProbe::empty(
                Connection::Unauthorized,
                vec![
                    "Authorize USB debugging on the device; no property requests were made.".into(),
                ],
            ),
            _ => DeviceProbe::empty(
                Connection::Offline,
                vec!["ADB device is not ready; no property requests were made.".into()],
            ),
        };
    }

    let fastboot_count = runner
        .run("fastboot", &["devices"])
        .ok()
        .filter(|o| o.success)
        .map(|o| parse_fastboot_count(&o.stdout))
        .unwrap_or(0);
    match fastboot_count {
        0 => DeviceProbe::empty(
            Connection::Disconnected,
            vec!["No available device. Connect a phone to use Android diagnostics.".into()],
        ),
        1 => DeviceProbe::empty(
            Connection::Fastboot,
            vec!["Fastboot connected; model and unlock status are not verified in P001.".into()],
        ),
        _ => DeviceProbe::empty(
            Connection::MultipleDevices,
            vec!["More than one Fastboot device detected; no device selected.".into()],
        ),
    }
}

fn probe_adb(runner: &dyn Runner) -> DeviceProbe {
    let prop = |key: &str| -> Option<String> {
        runner
            .run("adb", &["shell", "getprop", key])
            .ok()
            .filter(|o| o.success)
            .and_then(|o| sanitize_property(&o.stdout))
    };

    let model = prop("ro.product.model");
    let board = prop("ro.product.board");
    let device_name = prop("ro.product.device");
    let android_version = prop("ro.build.version.release");
    let lock = LockEvidence::new(
        prop("ro.boot.flash.locked"),
        prop("ro.boot.vbmeta.device_state"),
        prop("ro.boot.verifiedbootstate"),
    );

    let model_match = model.as_deref().is_some_and(|s| {
        let s = s.to_ascii_lowercase();
        s.contains("neo 2") || s.contains("z2352n")
    });
    let board_match = [board.as_deref(), device_name.as_deref()]
        .into_iter()
        .flatten()
        .any(|s| s.eq_ignore_ascii_case("P820F03"));

    let identity = if model_match && board_match {
        Identity::ReportedZ2352n
    } else {
        Identity::NotConfirmed
    };

    let mut notes = vec![
        "Property values are device-reported, not a hardware attestation.".into(),
        "Magisk/root is not evaluated; no su command was invoked.".into(),
    ];
    if identity == Identity::NotConfirmed {
        notes.push(
            "Device identity did not match BOTH the expected model and P820F03 board.".into(),
        );
    }
    if lock.assessment == LockAssessment::Inconsistent {
        notes.push("Lock/AVB properties conflict. Do not infer unlocked state.".into());
    }
    DeviceProbe {
        connection: Connection::Adb,
        identity,
        model,
        board,
        device_name,
        android_version,
        lock,
        notes,
    }
}

fn sanitize_property(value: &str) -> Option<String> {
    let clean: String = value
        .trim()
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .take(PROPERTY_LIMIT)
        .collect();
    if clean.is_empty() { None } else { Some(clean) }
}

fn parse_adb_states(value: &str) -> Vec<String> {
    value
        .lines()
        .filter_map(|line| {
            let (serial, state) = line.trim().split_once('\t')?;
            if serial.is_empty() {
                return None;
            }
            let state = state.split_whitespace().next()?;
            if matches!(
                state,
                "device" | "unauthorized" | "offline" | "recovery" | "sideload"
            ) {
                Some(state.to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn parse_fastboot_count(value: &str) -> usize {
    value
        .lines()
        .filter(|line| {
            let mut fields = line.split_whitespace();
            matches!((fields.next(), fields.next()), (Some(_), Some("fastboot")))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeRunner {
        values: HashMap<String, ProcessOutput>,
    }

    impl FakeRunner {
        fn insert(&mut self, command: &str, response: &str) {
            self.values.insert(
                command.into(),
                ProcessOutput {
                    success: true,
                    stdout: response.into(),
                },
            );
        }
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, String> {
            self.values
                .get(&format!("{} {}", program, args.join(" ")))
                .cloned()
                .ok_or_else(|| "not available".into())
        }
    }

    #[test]
    fn parses_adb_without_serials() {
        assert_eq!(
            parse_adb_states("List of devices attached\nSECRET_SERIAL\tdevice product:x\n"),
            vec!["device"]
        );
    }

    #[test]
    fn multiple_devices_fail_closed() {
        let mut fake = FakeRunner::default();
        fake.insert(
            "adb devices",
            "List of devices attached\nA\tdevice\nB\tdevice\n",
        );
        let result = probe(&fake);
        assert_eq!(result.connection, Connection::MultipleDevices);
        assert!(result.model.is_none());
    }

    #[test]
    fn unauthorized_device_not_probed() {
        let mut fake = FakeRunner::default();
        fake.insert("adb devices", "List of devices attached\nA\tunauthorized\n");
        assert_eq!(probe(&fake).connection, Connection::Unauthorized);
    }

    #[test]
    fn reported_unlocked_requires_consistent_triplet() {
        assert_eq!(
            classify_lock(Some("0"), Some("unlocked"), Some("orange")),
            LockAssessment::ReportedUnlocked
        );
        assert_eq!(
            classify_lock(Some("0"), Some("unlocked"), Some("green")),
            LockAssessment::Inconsistent
        );
        assert_eq!(
            classify_lock(Some("1"), Some("locked"), Some("green")),
            LockAssessment::ReportedLocked
        );
    }

    #[test]
    fn missing_properties_not_called_unlocked() {
        assert_eq!(
            classify_lock(Some("0"), None, Some("orange")),
            LockAssessment::InsufficientEvidence
        );
    }

    #[test]
    fn verifies_model_and_board_together() {
        let mut fake = FakeRunner::default();
        fake.insert(
            "adb devices",
            "List of devices attached\nSENSITIVE\tdevice\n",
        );
        fake.insert("adb shell getprop ro.product.model", "nubia Neo 2 5G\n");
        fake.insert("adb shell getprop ro.product.board", "P820F03\n");
        fake.insert("adb shell getprop ro.product.device", "P820F03\n");
        fake.insert("adb shell getprop ro.build.version.release", "13\n");
        fake.insert("adb shell getprop ro.boot.flash.locked", "0\n");
        fake.insert(
            "adb shell getprop ro.boot.vbmeta.device_state",
            "unlocked\n",
        );
        fake.insert("adb shell getprop ro.boot.verifiedbootstate", "orange\n");
        let result = probe(&fake);
        assert_eq!(result.identity, Identity::ReportedZ2352n);
        assert_eq!(result.lock.assessment, LockAssessment::ReportedUnlocked);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("SENSITIVE"));
    }

    #[test]
    fn unknown_identity_never_promoted() {
        let mut fake = FakeRunner::default();
        fake.insert("adb devices", "List of devices attached\nA\tdevice\n");
        fake.insert("adb shell getprop ro.product.model", "Some Neo 2 phone\n");
        fake.insert("adb shell getprop ro.product.board", "DIFFERENT\n");
        assert_eq!(probe(&fake).identity, Identity::NotConfirmed);
    }

    #[test]
    fn sanitizes_control_sequences() {
        assert_eq!(
            sanitize_property("hello\u{1b}[31m\n"),
            Some("hello[31m".into())
        );
        assert_eq!(sanitize_property("    "), None);
    }

    #[test]
    fn fastboot_is_only_detected() {
        let mut fake = FakeRunner::default();
        fake.insert("fastboot devices", "SERIAL\tfastboot\n");
        let result = probe(&fake);
        assert_eq!(result.connection, Connection::Fastboot);
        assert_eq!(result.lock.assessment, LockAssessment::InsufficientEvidence);
    }
}
