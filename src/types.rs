use std::fmt::Display;

use bitmask_enum::bitmask;
use clap::{Parser, ValueEnum};
use strum::FromRepr;

use crate::swion_result::SwionResult;

/// This represents a command type (first encoded byte in sci frame)
/// plus its command variant (third encoded byte in sci frame)
/// packed into a u16 with the high byte being the command type
/// and the low byte being the variant.
#[derive(Debug, FromRepr)]
#[repr(u16)]
pub enum CommandType {
    SysReadFirmwareVersion = 0x0001,
    SysReadSerialNumber = 0x0002,
    // Used without any arguments. No clue how to continue after this
    SysStartFirmwareUpdate = 0x0004,
    SysReadFeatureFlags = 0x0006,
    SysReadFirmwareBuildId = 0x0008,
    SysReadProductStringApplication = 0x0009,
    SysReadProductStringBootloader = 0x000A,
    SysReadHeapStatistics = 0x0060,
    DeviceResetReboot = 0x0100,
    DeviceResetShutdown = 0x0101,
    DeviceResetStartup = 0x0102,
    TimeSet = 0x0200,
    TimeGet1 = 0x0201,
    TimeGet3 = 0x0203,
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
    KeyPress = 0x1300,
    KeyReleaseMaybe = 0x1301,
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
    Tools3ClearCryptoKeys = 0x3001,
    Tools3WritePwmValue = 0x3004,
    // These two are for Crypto and OAP keys. Not sure which one is which
    Tools3ReadCryptoKeyCount06 = 0x3006,
    Tools3ReadCryptoKeyCount07 = 0x3007,
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
    CRACapabilityRead = 0x6700,
    CRALockKeyWrite = 0x6710,
    LockKeyReadAndAuth = 0x6711,
    LockKeyReset = 0x6712,
    LockKeyDeauth = 0x6713,
    CRALockKeyRead = 0x6715,
    CRALockKeyAuth = 0x6716,
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
        return format!("{}{}{}{}", read, write, i, p);
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
    MMICommands = 1,
    TTSCommands = 2,
    ExtendedNVMCommands = 4,
    SW09Commands = 8,
    PayloadCommands = 16,
    FeatureFlagCommands = 32,
    ADPCMAudioCommands = 64,
    LockKeyCommands = 128,
    LockKeyCRACommands = 256,
    ExtendedLogCommands = 512,
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
    pub payloadRequest: u16,
    pub payloadResponse: u16,
}

impl Display for CRACapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Flags: {}", self.flags)?;
        writeln!(f, "Payload Request: {}", self.payloadRequest)?;
        writeln!(f, "Payload Response: {}", self.payloadResponse)?;

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
    pub result: SwionResult,
    pub data: Vec<u8>,
}

pub struct DeviceInfo {
    pub serial_number: String,
    pub firmware_version: String,
    pub feature_flags: FeatureFlag,
}

impl Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Serial Number: {}", self.serial_number)?;
        writeln!(f, "Firmware Version: {}", self.firmware_version)?;
        writeln!(f, "Feature Flags: {}", self.feature_flags)?;

        Ok(())
    }
}
