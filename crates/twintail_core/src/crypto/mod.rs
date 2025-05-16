pub mod assetbundle;
pub mod decrypt;
pub mod encrypt;

#[derive(Clone, Copy)]
pub enum EncryptSuiteValuesState {
    /// the provided number of encrypted/decrypted files are being serialized
    SerializeStart(usize),
    /// the provided number of values were serialized
    Serialize(usize),
    /// suite files have finished being encrypted/decrypted
    Finish,
}

#[derive(Clone, Copy)]
pub enum EncryptSuitePathState {
    /// files are being processed
    Process,
}

#[derive(Clone, Copy)]
pub enum DecryptSuitePathState {
    /// The provided number of files are being decrypted
    Start(usize),
    /// A file has been decrypted
    Decrypt,
    /// The decryption process has finished
    Finish,
}

#[derive(Clone, Copy)]
pub enum CryptAssetbundlePathState {
    /// Files in the provided path are being scanned
    Scan,
    /// The provided number of files are being encrypted/decrypted
    Crypt(usize),
    /// A file was encrypted or decrypted
    CryptFile,
    /// The crypt operation finished
    Finish,
}

#[derive(Clone, Copy)]
pub enum CryptState {
    NoState,
    EncryptSuiteValues(EncryptSuiteValuesState),
    EncryptSuitePath(EncryptSuitePathState),
    DecryptSuitePath(DecryptSuitePathState),
    AssetbundlePath(CryptAssetbundlePathState),
}

impl Default for CryptState {
    fn default() -> Self {
        Self::NoState
    }
}
