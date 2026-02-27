use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashResult {
    pub hash: [u8; 32],
}

impl HashResult {
    /// 将哈希值转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        self.hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// 从十六进制字符串解析哈希值
    pub fn from_hex(s: &str) -> Result<Self> {
        if s.len() != 64 {
            bail!("哈希字符串长度必须为 64 个字符，实际为 {}", s.len());
        }

        let mut hash = [0u8; 32];
        for i in 0..32 {
            let byte_str = &s[i * 2..i * 2 + 2];
            hash[i] = u8::from_str_radix(byte_str, 16)
                .with_context(|| format!("无效的十六进制字符: {}", byte_str))?;
        }

        Ok(HashResult { hash })
    }
}

impl fmt::Display for HashResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for HashResult {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        HashResult::from_hex(s)
    }
}

impl PartialEq<str> for HashResult {
    fn eq(&self, other: &str) -> bool {
        self.to_hex() == other
    }
}

impl PartialEq<&str> for HashResult {
    fn eq(&self, other: &&str) -> bool {
        self.to_hex() == *other
    }
}

impl Serialize for HashResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for HashResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        HashResult::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// 计算文件的 SHA256 校验和
pub fn compute_file_hash(path: &Path) -> Result<HashResult> {
    let file = File::open(path).with_context(|| format!("无法打开文件: {:?}", path))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(HashResult {
        hash: hasher.finalize().into(),
    })
}
