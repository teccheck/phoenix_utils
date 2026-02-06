use std::{
    error::Error,
    fmt::{self, Display},
};

use bitmask_enum::bitmask;
use clap::{Parser, ValueEnum};
use serialport::{DataBits, Parity, StopBits};
use strum::{Display, FromRepr};

use crate::phoenix::swion_result::SwionResult;

/// This represents a command type (first encoded byte in sci frame)
/// plus its command variant (third encoded byte in sci frame)
/// packed into a u16 with the high byte being the command type
/// and the low byte being the variant.
#[derive(Debug, FromRepr, Clone)]
#[repr(u16)]
pub enum CommandType {
    SysReadFirmwareVersion = 0x0001,
    SysReadSerialNumber = 0x0002,
    SysUnknown0003 = 0x0003,
    // Used without any arguments. No clue how to continue after this
    SysStartFirmwareUpdate = 0x0004,
    SysReadFeatureFlags = 0x0006,
    SysWriteFeatureFlags = 0x007,
    SysReadFirmwareBuildId = 0x0008,
    // The following three are seemingly not for DE10A
    SysReadProductStringApplication = 0x0009,
    SysReadProductStringBootloader = 0x000A,
    SysReadHeapStatistics = 0x0060,

    BootReboot = 0x0100,
    BootShutdown = 0x0101,
    BootStartup = 0x0102,
    TimeSet = 0x0200,
    TimeGetUtc = 0x0201,
    TimeGetLocal = 0x0203,

    BatteryGetChargeState = 0x0301,

    MelodyPlaybackStart = 0x0500,
    MelodyPlaybackStop = 0x0501,

    AudioReadSegmentInfo = 0x0600,
    AudioSomethingSegmentBlock = 0x0601,
    AudioWriteSegmentBlock = 0x0602,
    AudioReadSegmentBlock = 0x0603,

    AnalogAdcRead = 0x1000,
    AnalogVref = 0x1001,
    AnalogTemperatureRead = 0x1003,

    ToolsBacklightNormalMode = 0x1103,
    ToolsBacklightTestMode = 0x1104,
    ToolsSequencerPlay06 = 0x1106,
    ToolsSequencerPlay07 = 0x1107,
    ToolsKeyClick = 0x1108,
    // Byte after variant is 2 for main and 1 for sub
    //ToolsClockToTpOff = 0x1109,
    //ToolsClockToTpOn = 0x1110,
    // Why is there so much overlap???
    //ToolsDigitalMonitorOff = 0x1109,
    //ToolsDigitalMonitorOn = 0x1110,
    // I'm sure about this one. Why is there ToolsClockX???
    // Ok, it seems like there's overlap, so this seems to be stateful?
    ToolsBeeperNormalMode = 0x1109,
    ToolsBeeperTestMode = 0x1110,
    ToolsLedNormalMode = 0x1111,
    ToolsLedTestMode = 0x1112,

    // Param 0 for normal
    DisplayTestMode = 0x1200,

    // These two are related and both named key press. They're propably press and release
    KeyRelease = 0x1300,
    KeyPress = 0x1301,

    StorageReadBlockPart = 0x1400,
    StorageWriteBlock = 0x1401,
    StorageDeleteBlock = 0x1402,
    StorageReadDirSize = 0x1410,
    StorageReadBlockInfo = 0x1411,
    StorageReadStatus = 0x1420,
    StorageUnknown30 = 0x1430,
    StorageExtNvmRead = 0x1440,
    StorageExtNvmWrite = 0x1441,
    StorageExtNvmReadDir = 0x1444,
    StorageTransactionStart = 0x1450,
    StorageTransactionTest = 0x1452,
    StorageTransactionConfirm = 0x1454,
    StorageTransactionAbort = 0x1458,
    StorageTransactionReadState = 0x145A,
    StorageOtpBackup = 0x1470,
    StorageOtpRestore = 0x1471,
    StorageExtNvmReadAlt = 0x14C0,
    StorageExtNvmWriteAlt = 0x14C1,

    ExtendedLogRead = 0x1900,

    KnobTurn = 0x1A00,

    // One of these unknown ones has to be write key
    CryptoUnknown3000 = 0x3000,
    CryptoClearKeys = 0x3001,
    CryptoWritePwmValue = 0x3004,
    CryptoReadKeyCount = 0x3006,
    CryptoReadKeyCountOAP = 0x3007,
    CryptoUnknown3008 = 0x3008,

    MessagesReadCount = 0x3100,
    MessagesRead01 = 0x3101, // Read in Monitor???
    MessagesClearPool = 0x3110,
    MessagesRead20 = 0x3120,

    FeatureFlagsActivate = 0x4300,
    FeatureFlagsReadUniqueId = 0x4301,
    FeatureFlagsReadEnabled = 0x4302,
    FeatureFlagsReadSupported = 0x4303,

    CalibrateAcc = 0x6000,

    GsmReadIMEI = 0x6109,
    GsmReadIMSI = 0x610A,
    GsmReadLocalAreaIdentity = 0x610B,
    GsmReadICCID = 0x610F,
    GsmSendAtCommand = 0x6113,
    GsmFotaCheck = 0x6124,
    GsmFotaUpdate = 0x6126,

    GpsEphemerisGetStorageCapacity = 0x6201,
    GpsEphemerisWrite = 0x6202,
    GpsReadVersion = 0x6205,

    SupportedPayload = 0x6300,

    TtsMessageCommand = 0x6500,
    // Actual command seems to be defined by fourth and fith byte (the two after the command variant byte)
    TtsFileCommand = 0x6501,
    TtsStartTestModeMaybe = 0x6502,
    TtsUnknown03 = 0x6503,
    // Also triggers startup
    TtsComModeEnter = 0x6510,
    TtsComModeExit = 0x6512,

    // Not sure why CapabilityRead is in here.
    // I think CRA means Challenge–Response Authentication here
    CapabilityRead = 0x6700,
    LockKeyCraWrite = 0x6710,
    LockKeyReadAndAuth = 0x6711,
    LockKeyReset = 0x6712,
    LockKeyDeauth = 0x6713,
    // Returns Serial number, a few unknown bytes and a counter as ascii chars.
    // The counter seems to count up with every 0x6714 command that is sent.
    // However none of the pagers, I tested started at a low counter (less than 100).
    // Most of them were at way more than 5000, even more than 10000.
    LockKeyUnknown6714 = 0x6714,
    LockKeyCraRead = 0x6715,
    LockKeyCraAuth = 0x6716,

    EuiEndpointRead = 0x6D01,

    DisplayReadLanguageList = 0x7000,
    DisplayReadSupportedMenus = 0x7001,
    DisplayReadSupportedFonts = 0x7002,
    // Byte after variant: 7: ricprog1, 11: ric500a, 13: tuning, 15: Testmode off
    DisplayActivateTestMode = 0x7006,
    DisplayUnlockTuning = 0x7007,

    TestMessageSend = 0x8001,

    Tools2RadioMode = 0x8102,
    // Byte after variant 0 for normal, 1 for shifted
    Tools2SetClockMode = 0x8104,
    Tools2SetTemperatureCompensation = 0x8109,
    Tools2SetCalibrationMode = 0x810A,
    Tools2EnterTestMode = 0x8110,
    Tools2OperationContinuous = 0x8111,
    Tools2AnalogMonitor = 0x811A,
    Tools2FlexTestMessage = 0x8180,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Parser)]
pub enum ResetType {
    Hardreset = 0,
    Softreset,
    BootupToHiddenMenu,
    BootupToTestMenu,
    BootupWithoutConfiguration,
    BootupToGsmTunnel,
    BootupToBootloader,
}

pub type StorageBlockId = u16;
pub type StorageBlockOffset = u16;
pub type StorageBlockLength = u16;
pub type StorageBlockVersion = u8;

#[bitmask(u8)]
pub enum StorageBlockPermissions {
    Read,
    Write,
    I,
    P,
}

impl StorageBlockPermissions {
    pub fn flag_string(&self) -> String {
        let read = if self.contains(StorageBlockPermissions::Read) {
            "R"
        } else {
            "-"
        };
        let write = if self.contains(StorageBlockPermissions::Write) {
            "W"
        } else {
            "-"
        };
        let i = if self.contains(StorageBlockPermissions::I) {
            "I"
        } else {
            "-"
        };
        let p = if self.contains(StorageBlockPermissions::P) {
            "P"
        } else {
            "-"
        };
        format!("{}{}{}{}", read, write, i, p)
    }
}

#[bitmask(u32)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum FeatureFlag {
    Idea = 1,
    MultiChannel = 2,
    RssiFeedback = 8,
    GuardProtected = 16,
    GuardConnected = 32,
    GuardBgr = 64,
    Glob = 128,
    BleFeedback = 256,
    BleHybridAlert = 512,
    Aes = 1024,
    Boskrypt = 2048,
    Pocsag = 4096,
    Flex = 8192,
    Sos = 16384,
    Tts = 32768,
    Dcsa = 65536,
}

impl Display for FeatureFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags: Vec<&'static str> = FeatureFlag::flags()
            .filter(|(_, f)| self.contains(*f))
            .map(|(n, _)| *n)
            .collect();
        write!(f, "{}", flags.join(", "))?;
        Ok(())
    }
}

#[bitmask(u16)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum CRACapabilityFlags {
    /// Seems to be commands related to menu structure and user interface
    MMICommands = 1,
    /// TTS stands for text to speach
    TTSCommands = 2,
    /// Extended commands for non volatile memory interaction
    ExtendedNVMCommands = 4,
    /// No idea what this is for
    SW09Commands = 8,
    /// Seems to be related to the two payload sizes
    PayloadCommands = 16,
    /// Allows to interact with feature flags
    FeatureFlagCommands = 32,
    /// ADPCM is a somewhat compressed audio format
    ADPCMAudioCommands = 64,
    /// Somewhat sane authentication via programming password
    LockKeyCommands = 128,
    /// Good? authentication via challenge response auth?
    LockKeyCRACommands = 256,
    /// Probably for reading system logs
    ExtendedLogCommands = 512,
    /// Allows transaction based storage operations
    TransactionCommands = 1024,
}

impl Display for CRACapabilityFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags: Vec<&'static str> = CRACapabilityFlags::flags()
            .filter(|(_, f)| self.contains(*f))
            .map(|(n, _)| *n)
            .collect();
        write!(f, "{}", flags.join(", "))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CRACapabilities {
    pub flags: CRACapabilityFlags,
    pub payload_request: u16,
    pub payload_response: u16,
}

impl Display for CRACapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Flags: {}", self.flags)?;
        writeln!(f, "Payload Request: {}", self.payload_request)?;
        writeln!(f, "Payload Response: {}", self.payload_response)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct StorageBlockInfo {
    pub id: StorageBlockId,
    pub length: StorageBlockLength,
    pub version: StorageBlockVersion,
    pub permissions: StorageBlockPermissions,
}

#[derive(Debug)]
pub struct PartialStorageBlock {
    pub id: StorageBlockId,
    pub offset: StorageBlockOffset,
    pub length: StorageBlockLength,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct ReadStorageBlock {
    pub id: StorageBlockId,
    pub offset: StorageBlockOffset,
    pub length: StorageBlockLength,
}

pub struct DeviceInfo {
    pub serial_number: String,
    pub firmware_version: String,
    pub firmware_build_id: String,
    pub feature_flags: FeatureFlag,
}

impl Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Serial Number: {}", self.serial_number)?;
        writeln!(f, "Firmware Version: {}", self.firmware_version)?;
        writeln!(f, "Firmware Build Id: {}", self.firmware_build_id)?;
        writeln!(f, "Feature Flags: [{}]", self.feature_flags)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AuthError {
    pub result: SwionResult,
    pub remaining_attempts: u8,
    pub locked_until_day: u8,
    pub locked_until_month: u8,
    pub locked_until_year: u16,
    pub enhanced_protection: u8,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Authentication failed. Attempts remaining: {}, Enhanced Protection: {}",
            self.remaining_attempts, self.enhanced_protection
        )?;

        if self.locked_until_day != 0xFF
            && self.locked_until_month != 0xFF
            && self.locked_until_year != 0xFFFF
        {
            write!(
                f,
                "Locked until: {}-{}-{}",
                self.locked_until_year, self.locked_until_month, self.locked_until_day
            )?;
        }

        Ok(())
    }
}

impl Error for AuthError {}

#[derive(Debug, Clone)]
pub struct FeatureFlagNotFoundError {
    pub flag_name: String,
}

impl fmt::Display for FeatureFlagNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Feature flag not valid: {}", self.flag_name,)?;
        Ok(())
    }
}

impl Error for FeatureFlagNotFoundError {}

#[derive(Debug, Clone)]
pub struct InvalidResponseTypeError {
    pub type_required: u16,
    pub type_actual: u16,
}

impl fmt::Display for InvalidResponseTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Response had invalid type. Should have been {:X?}, but was {:X?}",
            self.type_required, self.type_actual,
        )?;
        Ok(())
    }
}

impl Error for InvalidResponseTypeError {}

impl InvalidResponseTypeError {
    pub fn new(type_required: u16, type_actual: u16) -> InvalidResponseTypeError {
        InvalidResponseTypeError {
            type_required,
            type_actual,
        }
    }
}

/// Not sure what they mean
/// Also known as device generation
#[derive(Debug, Display, FromRepr)]
#[repr(u8)]
pub enum DeviceType {
    /// Some kind of default/error value (perhaps unknown)
    None = 0x00,

    /// Unknown device type (maybe older DE09A hardware?)
    /// Answers with 0x55 to the initial handshake
    B = 0x55,

    /// Seems to be the type for all DE10A Hardware from Firmware 3.94 to 4.90
    /// Uses serial config 2 (57600 Baud, 8 Data bits, 1 Stop bit, Parity 0)
    /// Answers with 0x56 to the initial handshake
    DE10A,

    /// Unknown device type
    /// Uses serial config 1 (460800 Baud, 8 Data bits, 1 Stop bit, Parity 0)
    /// Supports encrypted dump
    /// Answers with 0x57 to the initial handshake
    D,

    /// Unknown device type
    /// Supports encrypted dump
    /// Answers with 0x58 to the initial handshake
    E,

    /// Unknown device type
    /// Uses serial config 1 (460800 Baud, 8 Data bits, 1 Stop bit, Parity 0)
    /// Answers with 0x59 to the initial handshake
    F,
}

impl DeviceType {
    pub fn parse(result: u8) -> DeviceType {
        DeviceType::from_repr(result).unwrap_or(DeviceType::None)
    }
}

pub struct SerialConfig {
    pub baudrate: u32,
    pub databits: DataBits,
    pub parity: Parity,
    pub stopbits: StopBits,
}

pub const SERIAL_CONFIG_1: SerialConfig = SerialConfig {
    baudrate: 460800,
    databits: DataBits::Eight,
    stopbits: StopBits::One,
    parity: Parity::None,
};

pub const SERIAL_CONFIG_2: SerialConfig = SerialConfig {
    baudrate: 57600,
    databits: DataBits::Eight,
    stopbits: StopBits::One,
    parity: Parity::None,
};
